use bastion::prelude::*;
use futures::future::lazy;

#[derive(Debug)]
struct DemoMessage;

fn main() {
    Bastion::init();
    Bastion::start();

    let mut receiver_ref = None;

    Bastion::supervisor(|sp| {
        let sp = sp.with_strategy(SupervisionStrategy::OneForOne);
        receiver_ref = Some(
            sp.children_ref(move |children| {
                children
                    .with_name("Receiver")
                    .with_exec(move |ctx| async move {
                        loop {
                            if let Some(msg) = ctx.try_recv().await {
                                msg! { msg,
                                    msg: DemoMessage =!> {
                                        eprintln!("DemoMessage!");
                                        answer!(ctx, 42u8).unwrap();
                                    };
                                    _:_ => panic!("Invalid message received");
                                }
                            } else {
                                eprintln!("Idling around");
                            }
                        }
                    })
            })
            .elems()[0]
                .clone(),
        );
        sp
    })
    .unwrap();

    run!(async {
        msg! { receiver_ref
        .unwrap()
        .ask_anonymously(DemoMessage)
        .unwrap()
        .await
        .unwrap(),
            response: u8 => {
                assert_eq!(response, 42);
                eprintln!("Response = {:?}", response);
            };
            _:_ => panic!("Invalid answer received");
        }
    });

    eprintln!("Blocking...");

    Bastion::block_until_stopped();
}
