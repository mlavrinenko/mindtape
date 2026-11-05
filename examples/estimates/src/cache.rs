use crate::loader::HarvestCache;

pub fn harvest() -> HarvestCache {
    HarvestCache
}

pub fn warm(_cache: &HarvestCache) {}

pub fn warm_twice(cache: &HarvestCache) {
    warm(cache);
}
