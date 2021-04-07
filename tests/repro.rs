use std::sync::Arc;

use pollster::block_on;
use switchyard::{
    threads::{single_pool_two_to_one, thread_info},
    JoinHandle, Switchyard,
};

#[derive(PartialEq, Eq, Debug)]
struct ResourceA {
    number: i32,
}
impl ResourceA {
    async fn load(_name: String, _yard: Arc<Switchyard<()>>) -> Self {
        Self { number: 1 }
    }
}

fn spawn(yard: Arc<Switchyard<()>>, name: String) -> JoinHandle<Arc<ResourceA>> {
    let _yard = yard.clone();
    yard.spawn(0, 0, async move {
        let resource = ResourceA::load(name.clone(), _yard);

        eprintln!("awaiting {}", name);
        let resource = Arc::new(resource.await);
        eprintln!("done awaiting {}", name);

        resource
    })
}

#[test]
fn repro() {
    let yard = Arc::new(Switchyard::new(1, single_pool_two_to_one(thread_info(), None), || ()).unwrap());

    let yard_a = yard.clone();
    let a = std::thread::spawn(move || spawn(yard_a, "abv".into()));
    let yard_b = yard;
    let b = std::thread::spawn(move || spawn(yard_b, "def".into()));

    eprintln!("A");

    let a = a.join().unwrap();
    let b = b.join().unwrap();

    eprintln!("B");

    let a = block_on(a);
    let b = block_on(b);
    assert_eq!(a, b);

    eprintln!("C");
}
