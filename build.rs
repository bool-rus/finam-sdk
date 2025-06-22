fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("running prost codegen");
    tonic_build::configure().build_server(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        //.client_attribute(".", "#[derive(derive_more::From, derive_more::Into)]")
        .include_file("mod.rs")
        .compile_protos(
            &[
                "protos/grpc/tradeapi/v1/accounts/accounts_service.proto",
                "protos/grpc/tradeapi/v1/assets/assets_service.proto",
                "protos/grpc/tradeapi/v1/auth/auth_service.proto",
                "protos/grpc/tradeapi/v1/marketdata/marketdata_service.proto",
                "protos/grpc/tradeapi/v1/orders/orders_service.proto",
            ],
            &[
                "protos"
            ],
        )?;
    Ok(())
}

