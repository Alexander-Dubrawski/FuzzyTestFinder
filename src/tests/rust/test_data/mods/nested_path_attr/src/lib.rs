#[path("thread_files")]
mod thread {
    #[path("tls.rs")]
    mod local_data;

    #[path("process_files")]
    mod pf {
        #[path("pid.rs")]
        mod local_data;
    }
}