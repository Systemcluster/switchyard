use std::{sync::Arc, time::Duration};

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
    async fn load(_name: String, _store: Arc<Store>) -> Result<Self, ()>
    where
        Self: Sized,
    {
        std::thread::sleep(Duration::from_millis(100));
        Ok(Self { number: 1 })
    }
}

struct Store {
    tasks: Switchyard<()>,
}
impl Store {
    pub fn new() -> Self {
        Self {
            tasks: Switchyard::new(1, single_pool_two_to_one(thread_info(), None), || ()).unwrap(),
        }
    }
    fn get(self: &Arc<Self>, name: String) -> JoinHandle<Arc<ResourceA>> {
        let _self = self.clone();
        self.tasks.spawn(0, 0, async move {
            eprintln!("loading {}", name);
            let resource = ResourceA::load(name.clone(), _self);
            eprintln!("awaiting {}", name);
            let resource = Arc::new(resource.await.unwrap_or_else(|error| {
                panic!("loading resource {} failed: {:#?}", name, error);
            }));
            eprintln!("done awaiting {}", name);
            resource
        })
    }
}

#[test]
fn repro() {
    let store = Arc::new(Store::new());

    let storea = store.clone();
    let a = std::thread::spawn(move || storea.get("abv".into()));
    let storeb = store;
    let b = std::thread::spawn(move || storeb.get("def".into()));

    eprintln!("A");

    let a = a.join().unwrap();
    let b = b.join().unwrap();

    eprintln!("B");

    let a = block_on(a);
    let b = block_on(b);
    assert_eq!(a, b);

    eprintln!("C");
}
