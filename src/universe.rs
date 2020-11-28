use author;
use remnant;
use std::collections::HashMap;

/// A Universe contains two collections:
///    * The known author identifiers and their public keys
///    * The known remnants
///
/// In addition, a Universe also contains information about the author
/// to use for the currently running process.
#[derive(Debug)]
pub struct Universe {
    /// The authors we can verify/validate.
    authors: HashMap<author::AuthorId, author::PartialAuthor>,

    /// The remnants we know about.
    remnants: HashMap<remnant::NodeId, remnant::Remnant>,

    /// The author we act as.
    author: author::Author,
}

impl Universe {
    pub fn new(author: &author::Author) -> Universe {
        Universe {
            authors: HashMap::new(),
            remnants: HashMap::new(),
            author: author.clone(),
        }
    }
}
