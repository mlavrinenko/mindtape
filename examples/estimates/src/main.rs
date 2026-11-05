mod cache;
mod loader;

use cache::harvest;
use cache::warm;

fn main() {
    let cache = harvest();
    warm(&cache);
    warm(&cache);
}
