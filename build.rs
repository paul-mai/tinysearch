use bincode;
use cuckoofilter::{self, CuckooFilter, ExportedCuckooFilter};
use walkdir::{DirEntry, WalkDir};

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[path = "src/types.rs"]
mod types;

use crate::types::Storage;

fn main() -> Result<(), Box<Error>> {
    let filters = build(".".to_string())?;
    let storage = Storage::from(filters);
    fs::write("storage", storage.to_bytes()?)?;
    Ok(())

    // let words = vec!["foo", "bar", "xylophone", "milagro"];
    // let mut filter = cuckoofilter::CuckooFilter::new();

    // let mut insertions = 0;
    // for s in &words {
    //     filter.add(s).unwrap();
    //     insertions += 1;
    // }

    // Export the fingerprint data stored in the filter,
    // along with the filter's current length.
    // let store: ExportedCuckooFilter = filter.export();
    // let encoded: Vec<u8> = bincode::serialize(&store).unwrap();
    // let encoded: Vec<u8> = bincode::serialize(&filters).unwrap();
    // fs::write("store", encoded);
    // Ok(())
}

fn build(corpus_path: String) -> Result<HashMap<PathBuf, CuckooFilter<DefaultHasher>>, Box<Error>> {
    let posts = prepare_posts(corpus_path)?;
    generate_filters(posts)
}

// Read all posts and generate Bloomfilters from them.
#[no_mangle]
pub fn generate_filters(
    posts: HashMap<PathBuf, String>,
) -> Result<HashMap<PathBuf, CuckooFilter<DefaultHasher>>, Box<Error>> {
    // Create a dictionary of {"post name": "lowercase word set"}. split_posts =
    // {name: set(re.split("\W+", contents.lower())) for name, contents in
    // posts.items()}
    let split_posts: HashMap<PathBuf, HashSet<String>> = posts
        .into_iter()
        .map(|(post, content)| {
            (
                post,
                content
                    .split_whitespace()
                    .map(str::to_lowercase)
                    .collect::<HashSet<String>>(),
            )
        })
        .collect();

    // At this point, we have a dictionary of posts and a normalized set of
    // words in each. We could do more things, like stemming, removing common
    // words (a, the, etc), but we’re going for naive, so let’s just create the
    // filters for now:
    let mut filters = HashMap::new();
    for (name, words) in split_posts {
        // let mut filter = Cuckoofilter::with_capacity(words.len() as u32);
        let mut filter = cuckoofilter::CuckooFilter::new();
        for word in words {
            filter.add(&word);
        }
        filters.insert(name, filter);
    }
    Ok(filters)
}

fn is_markdown(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".md"))
        .unwrap_or(false)
}

// prepares the files in the given directory to be consumed by the generator
pub fn prepare_posts(dir: String) -> Result<HashMap<PathBuf, String>, Box<Error>> {
    let mut posts: HashMap<PathBuf, String> = HashMap::new();
    let walker = WalkDir::new(dir).into_iter();
    for entry in walker.filter_entry(|e| is_markdown(e)) {
        let entry = entry?;
        println!("Analyzing {}", entry.path().display());

        let mut post = File::open(entry.path())?;
        let mut contents = String::new();
        post.read_to_string(&mut contents)?;
        posts.insert(entry.into_path(), contents);
    }
    Ok(posts)
}
