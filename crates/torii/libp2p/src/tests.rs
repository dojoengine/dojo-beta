#[cfg(test)]
mod test {
    use std::error::Error;

    use futures::StreamExt;

    use crate::client::{Command, RelayClient};

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    // This tests subscribing to a topic and receiving a message
    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_client_messaging() -> Result<(), Box<dyn Error>> {
        use std::time::Duration;

        use tokio::time::sleep;
        use tokio::{self, select};

        use crate::server::Relay;

        let _ = tracing_subscriber::fmt()
            .with_env_filter("torii::relay::client=debug,torii::relay::server=debug")
            .try_init();
        // Initialize the relay server
        let mut relay_server: Relay = Relay::new(9090, 9091, None, None)?;
        tokio::spawn(async move {
            relay_server.run().await;
        });

        // Initialize the first client (listener)
        let mut client = RelayClient::new("/ip4/127.0.0.1/tcp/9090".to_string())?;
        tokio::spawn(async move {
            client.event_loop.run().await;
        });

        // Give some time for the client and server to start up
        sleep(Duration::from_secs(1)).await;

        client.command_sender.unbounded_send(Command::Subscribe("mawmaw".to_string()))?;
        sleep(Duration::from_secs(1)).await;
        client
            .command_sender
            .unbounded_send(Command::Publish("mawmaw".to_string(), "mimi".as_bytes().to_vec()))?;

        loop {
            select! {
                event = client.message_receiver.next() => {
                    if let Some(message) = event {
                        println!("Received message from {:?} with id {:?}: {:?}", message.source, message.message_id, message);
                        return Ok(());
                    }
                }
                _ = sleep(Duration::from_secs(5)) => {
                    println!("Test Failed: Did not receive message within 5 seconds.");
                    return Err("Timeout reached without receiving a message".into());
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test]
    async fn test_client_connection_wasm() -> Result<(), Box<dyn Error>> {
        use futures::future::{select, Either};
        use wasm_bindgen_futures::spawn_local;

        tracing_wasm::set_as_global_default();

        let _ = tracing_subscriber::fmt().with_env_filter("torii_libp2p=debug").try_init();
        // Initialize the first client (listener)
        // Make sure the cert hash is correct - corresponding to the cert in the relay server
        let mut client = RelayClient::new(
            "/ip4/127.0.0.1/udp/9091/webrtc-direct/certhash/\
             uEiD6v3wzt8XU3s3SqgNSBJPvn9E0VMVFm8-G0iSEsIIDxw"
                .to_string(),
        )?;

        spawn_local(async move {
            client.event_loop.run().await;
        });

        // Give some time for the client to start up
        let _ = wasm_timer::Delay::new(std::time::Duration::from_secs(10)).await;

        client.command_sender.unbounded_send(Command::Subscribe("mawmaw".to_string()))?;
        let _ = wasm_timer::Delay::new(std::time::Duration::from_secs(1)).await;
        client
            .command_sender
            .unbounded_send(Command::Publish("mawmaw".to_string(), "mimi".as_bytes().to_vec()))?;

        let timeout = wasm_timer::Delay::new(std::time::Duration::from_secs(5));
        let message_future = client.message_receiver.next();

        match select(message_future, timeout).await {
            Either::Left((Some(_message), _)) => {
                println!("Test OK: Received message within 5 seconds.");
                Ok(())
            }
            _ => {
                println!("Test Failed: Did not receive message within 5 seconds.");
                Err("Timeout reached without receiving a message".into())
            }
        }
    }
}
