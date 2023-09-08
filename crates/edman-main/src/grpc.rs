pub mod chrome_extension {
    #![allow(non_snake_case)]
    tonic::include_proto!("chrome_extension");
}

pub mod ui {
    #![allow(non_snake_case)]
    tonic::include_proto!("ui");
}

pub mod config {
    #![allow(non_snake_case)]
    tonic::include_proto!("config");
}
