use std::{sync::{Arc, OnceLock}, cell::OnceCell};
use http_cache_reqwest::{MokaCache, MokaCacheBuilder, MokaManager};
use tokio::sync::RwLock;


// use std::cell::OnceCell;
// use moka::future::{Cache};

// lazy_static::lazy_static! {
//     // static DB_POOL: OnceCell<Cache> = OnceCell::new();
//     pub static MOKA_CACHE: MokaCache<String, Arc<Vec<u8>>> = {
//         MokaCache::new(250)
//     };
// }

pub static MOKA_CACHE: RwLock<MokaCache<String, Arc<Vec<u8>>>> = RwLock::new(MokaCache::new(250));

// pub static MOKA_CACHE: MokaCache<String, Arc<Vec<u8>>> = {
//     MokaCache::new(250)
// };
// // let moka_cache = Cache::new(250);