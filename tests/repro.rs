use std::{any::Any, sync::Arc, time::Duration};

use async_trait::async_trait;
use pollster::block_on;
use switchyard::{
    threads::{single_pool_two_to_one, thread_info},
    JoinHandle, Switchyard,
};

#[async_trait]
trait Resource: Any + Send + Sync {
    async fn load(name: String, store: Arc<Store>) -> Result<Self, ()>
    where
        Self: Sized;
}

#[derive(PartialEq, Eq, Debug)]
struct ResourceA {
    number: i32,
}
#[async_trait]
impl Resource for ResourceA {
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
    fn get<T: Resource>(self: &Arc<Self>, name: String) -> JoinHandle<Arc<T>> {
        let _self = self.clone();
        self.tasks.spawn(0, 0, async move {
            eprintln!("loading {}", name);
            let resource = T::load(name.clone(), _self);
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
    let a = std::thread::spawn(move || storea.get::<ResourceA>("abv".into()));
    let storeb = store;
    let b = std::thread::spawn(move || storeb.get::<ResourceA>("def".into()));

    eprintln!("A");

    let a = a.join().unwrap();
    let b = b.join().unwrap();

    eprintln!("B");

    let a = std::thread::spawn(move || block_on(a));
    let b = std::thread::spawn(move || block_on(b));

    eprintln!("C");

    let a = a.join().unwrap();
    let b = b.join().unwrap();
    assert_eq!(a, b);
}
