#[path("thread_files")]
mod thread {
    #[path("tls.rs")]
    mod local_data;

    #[path("process_files")]
    mod pf {
        #[path("pid.rs")]
        mod local_data;
        mod hello;
    }

    mod foo {}
}

// HashMap;

// // IF file contains mods, map to dict

// thread -> p/lib
// thread::local_data -> p/tls.rs
// thread::pf -> p/process_files
// thread::pf::local_data -> p/pid.rs

// // iterate over key value
// parse file with key as path and value as file path

// inside the file build dict again and continue