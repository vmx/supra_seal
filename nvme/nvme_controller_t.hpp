// Copyright Supranational LLC

#ifndef __NVME_CONTROLLER_T_HPP__
#define __NVME_CONTROLLER_T_HPP__

class nvme_controller_t {
  friend nvme_controllers_t;
  friend nvme_namespace_t;
  friend nvme_qpair_t;

public:
  static const size_t queue_size = 1024;
  
private:
  std::string                   name;
  struct spdk_nvme_ctrlr*       ctrlr;
  std::vector<nvme_namespace_t> namespaces;
  std::vector<nvme_qpair_t*>    qpairs;

public:
  nvme_controller_t(const char* _name,
                    struct spdk_nvme_ctrlr* _ctrlr) {
    name = _name;
    ctrlr = _ctrlr;
  }

  std::string get_name() {
    return name;
  }

  size_t get_sector_count(size_t ns_id) {
    return namespaces[ns_id].get_sector_count();
  }

  size_t get_page_count(size_t ns_id) {
    return namespaces[ns_id].get_page_count();
  }

  void cleanup() {
    for (auto it: qpairs) {
      it->cleanup();
      delete(it);
    }
  }
  
  int register_namespaces() {
    // Each controller has one or more namespaces.  An NVMe namespace is
    // basically equivalent to a SCSI LUN.  The controller's IDENTIFY data
    // tells us how many namespaces exist on the controller.  For Intel(R)
    // P3X00 controllers, it will just be one namespace.
    // Note that in NVMe, namespace IDs start at 1, not 0.
   
    for (int nsid = spdk_nvme_ctrlr_get_first_active_ns(ctrlr); nsid != 0;
         nsid = spdk_nvme_ctrlr_get_next_active_ns(ctrlr, nsid)) {
      struct spdk_nvme_ns* ns = spdk_nvme_ctrlr_get_ns(ctrlr, nsid);
      if (ns == NULL) {
        continue;
      }
      if (!spdk_nvme_ns_is_active(ns)) {
        continue;
      }
      namespaces.emplace(namespaces.end(), ns);
      //auto it = namespaces.emplace(namespaces.end(), ns);
      //it->print();
    }
    return 0;
  }

  int alloc_qpairs(size_t count) {
    // Allocate an I/O qpair that we can use to submit read/write requests
    // to namespaces on the controller.  NVMe controllers typically support
    // many qpairs per controller.  Any I/O qpair allocated for a controller
    // can submit I/O to any namespace on that controller.
     
    // The SPDK NVMe driver provides no synchronization for qpair accesses -
    // the application must ensure only a single thread submits I/O to a
    // qpair, and that same thread must also check for completions on that
    // qpair.  This enables extremely efficient I/O processing by making all
    // I/O operations completely lockless.
    struct spdk_nvme_io_qpair_opts opts;
    spdk_nvme_ctrlr_get_default_io_qpair_opts(ctrlr, &opts, sizeof(opts));
    opts.io_queue_requests = queue_size;
    //printf("Allocating %d io_queue_requests\n", opts.io_queue_requests);
    opts.delay_cmd_submit = true;

    for (size_t i = 0; i < count; i++) {
      struct spdk_nvme_qpair* qpair =
        spdk_nvme_ctrlr_alloc_io_qpair(ctrlr, &opts, sizeof(opts));
      if (qpair == NULL) {
        return 1;
      }
      qpairs.push_back(new nvme_qpair_t(qpair));
    }
    return 0;
  }

  static void io_complete(void *arg, const struct spdk_nvme_cpl *completion) {
    nvme_io_tracker_t* io = (nvme_io_tracker_t*)arg;

    // See if an error occurred. If so, display information
    // about it, and set completion value so that I/O
    // caller is aware that an error occurred.
    if (spdk_nvme_cpl_is_error(completion)) {
      spdk_nvme_qpair_print_completion(io->qpair->get_qpair(),
                                       (struct spdk_nvme_cpl *)completion);
      fprintf(stderr, "I/O error status: %s\n",
              spdk_nvme_cpl_get_status_string(&completion->status));
      fprintf(stderr, "I/O failed, aborting run\n");
      exit(1);
    }
    if (io->completion_cb) {
      if (io->completion_cb(io->completion_arg) != 0) {
        fprintf(stderr, "I/O callback failed, aborting run\n");
        exit(1);
      }
    }
  }

  // Note: user needs to call process_completions to poll for completed
  // io. The buffer must be reserved in advance using reserve_buf. The
  // buffer will be returned to the pool when the IO completes.
  // buf_gid is a global id.
  int write(nvme_io_tracker_t* io, size_t ns_id, size_t qpair_id, size_t offset,
            completion_cb_t cb = nullptr, void *cb_arg = nullptr) {
    io->ns = &namespaces[ns_id];
    io->qpair = qpairs[qpair_id];
    io->completion_cb = cb;
    io->completion_arg = cb_arg;

    uint32_t sector_size = io->ns->get_sector_size();
    uint32_t sectors_per_block = BLOCK_SIZE / sector_size;
    
    SPDK_ERROR(spdk_nvme_ns_cmd_write(io->ns->get_ns(),
                                      io->qpair->get_qpair(),
                                      io->buf,
                                      offset * sectors_per_block, // LBA start
                                      io->len() / sector_size,
                                      io_complete,
                                      io, 0));
    io->qpair->incr_ops();
    return 0;
  }

  // Note: user needs to call process_completions to poll for completed
  // io. The buffer must be reserved in advance using reserve_buf. The
  // buffer will be returned to the pool when the IO completes.
  // buf_id is a global id.
  int read(nvme_io_tracker_t* io, size_t ns_id, size_t qpair_id, size_t offset,
           completion_cb_t cb = nullptr, void *cb_arg = nullptr) {
    io->ns = &namespaces[ns_id];
    io->qpair = qpairs[qpair_id];
    io->completion_cb = cb;
    io->completion_arg = cb_arg;
    
    uint32_t sector_size = io->ns->get_sector_size();
    uint32_t sectors_per_block = BLOCK_SIZE / sector_size;

    SPDK_ERROR(spdk_nvme_ns_cmd_read(io->ns->get_ns(),
                                     io->qpair->get_qpair(),
                                     io->buf,
                                     offset * sectors_per_block, // LBA start
                                     io->len() / sector_size,
                                     io_complete,
                                     io, 0));
    io->qpair->incr_ops();
    return 0;
  }

  size_t get_outstanding_io_ops(size_t qpair) {
    return qpairs[qpair]->get_outstanding_io_ops();
  }

  int process_completions(size_t qpair) {
    return qpairs[qpair]->process_completions();

  }

  int process_all_completions(size_t qpair) {
    int completions = 0;
    while (qpairs[qpair]->get_outstanding_io_ops() > 0) {
      completions =+ qpairs[qpair]->process_completions();
    }
    return completions;
  }
};


class nvme_controllers_t  {
  struct spdk_nvme_transport_id trid = {};

  std::set<std::string>           allowed_nvme;
  std::vector<nvme_controller_t*> controllers;
  //size_t                     total_buffer_count;
  
  static bool probe_cb(void* cb_ctx,
                       const struct spdk_nvme_transport_id* trid,
                       struct spdk_nvme_ctrlr_opts* opts) {
    nvme_controllers_t* me = (nvme_controllers_t* )cb_ctx;
    if (me->allowed_nvme.find(trid->traddr) != me->allowed_nvme.end()) {
      printf("Attaching to %s\n", trid->traddr);
      return true;
    } else {
      printf("NOT Attaching to %s\n", trid->traddr);
      return false;
    }
  }

  static void attach_cb(void* cb_ctx,
                        const struct spdk_nvme_transport_id* trid,
                        struct spdk_nvme_ctrlr* ctrlr,
                        const struct spdk_nvme_ctrlr_opts* opts) {
    nvme_controllers_t* me = (nvme_controllers_t* )cb_ctx;
    
    //printf("Attached to %s\n", trid->traddr);
    
    // spdk_nvme_ctrlr is the logical abstraction in SPDK for an NVMe
    // controller.  During initialization, the IDENTIFY data for the
    // controller is read using an NVMe admin command, and that data
    // can be retrieved using spdk_nvme_ctrlr_get_data() to get
    // detailed information on the controller.  Refer to the NVMe
    // specification for more details on IDENTIFY for NVMe controllers.
    nvme_controller_t* controller = new nvme_controller_t(trid->traddr, ctrlr);
    controller->register_namespaces();
    me->controllers.push_back(controller);
  }
  
public:
  nvme_controllers_t(std::set<std::string> _allowed_nvme) {
    allowed_nvme = _allowed_nvme;
  }
  
  ~nvme_controllers_t() {
    for (auto it: controllers) {
      it->cleanup();
      delete it;
    }
  }

  size_t size() {
    return controllers.size();
  }

  nvme_controller_t &operator[](size_t i) {
    return *controllers[i];
  }

  // Remove controller from the list
  void remove(size_t i) {
    controllers.erase(controllers.begin() + i, controllers.begin() + i + 1);
  }
  
  int init(size_t qpair_count) {
    SPDK_ERROR(probe());
    SPDK_ERROR(alloc_qpairs(qpair_count));
    return 0;
  }

  int probe() {
    spdk_nvme_trid_populate_transport(&trid, SPDK_NVME_TRANSPORT_PCIE);
    snprintf(trid.subnqn, sizeof(trid.subnqn), "%s", SPDK_NVMF_DISCOVERY_NQN);
    
    int rc = spdk_nvme_probe(&trid, this, probe_cb, attach_cb, NULL);
    if (rc != 0) {
      fprintf(stderr, "spdk_nvme_probe() failed\n");
      return 1;
    }
    return 0;
  }

  int alloc_qpairs(size_t count) {
    int rc;
    for (auto it: controllers) {
      if ((rc = it->alloc_qpairs(count)) != 0) {
        return rc;
      }
    }
    return 0;
  }

  static bool sort_function(nvme_controller_t *i, nvme_controller_t *j) {
    return i->get_name() < j->get_name();
  }
  void sort() {
    std::sort(controllers.begin(), controllers.end(), sort_function);
  }
};

#endif
