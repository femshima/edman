use self::ui::edman_main_client::EdmanMainClient;

pub mod config {
    #![allow(non_snake_case)]
    tonic::include_proto!("config");
}

pub mod ui {
    #![allow(non_snake_case)]
    tonic::include_proto!("ui");
}

pub type Client = EdmanMainClient<transport::GrpcChannel>;
