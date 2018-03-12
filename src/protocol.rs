use author;
use remnant;

#[derive(Debug)]
pub enum Request {
    /// The first message sent. It contains the PartialAuthor of the
    /// connecting peer.
    Hello(author::PartialAuthor),

    /// The last messgae sent before a clean disconnect.
    Goodbye,

    /// Look up the public key for the specified author.
    LookupAuthor(author::AuthorId),

    /// Look up a remnant that the peer may know about.
    LookupRemnant(remnant::NodeId),

    /// Advertise a remnant by its Id.
    AdvertiseRemnant(remnant::NodeId),
}

#[derive(Debug)]
pub enum Response {
    /// The response to Hello is to send back our own ID and public key.
    Hello(author::PartialAuthor),

    /// A trivial response is okay for Goodbye.
    Goodbye,

    /// We respond to an author lookup request possibly with an author
    /// and possibly with nothing.
    LookupAuthor(Option<author::PartialAuthor>),

    /// We respond to a remnant lookup request possibly with the
    /// remnant and possibly with nothing.
    LookupRemnant(Option<remnant::Remnant>),

    /// A trivial response is okay for an advertisement.
    AdvertiseRemnant,
}

// What I've got above will work if both peers announce their entire
// library when either connects, but this doesn't make a lot of
// sense. Eventually, some sort of discovery based on a particular
// remnant will be needed. This will include discovery of decendants
// and querying longer chains of nodes all at once.
//
// However, for starters, we're going to just focus on being able to
// identify decendents. The form this takes can be pretty simple for
// now, but will need to be open ended.
//
// R
// D1 D2 D3 D4
//
// Parent(D3) = R
// NextSibling(D3) = D4
// PreviousSibling(D3) = D2
// FirstDecendant(R) = D1
// DecendantsHash(R) = Hash(Decendants(R))
// DecendantsHashAfter(R,D3) = Hash([D4])
//
// The tree can be crawled by performing a depth-first-search of the
// decendants and then siblings. The DecendantsHash can be used to
// detect whether or not new decendnats now exist under the root.
//
// ====
//
// Standing back, the most obvious way to store things is in a hash
// table, but uppon closer inspection, this actually poses some
// problems. In order to do synchronization, I need fast access to an
// ordered list of the entries. The best structure to use to
// synchronize this lists is almost certainly going to be a Merkle
// tree anchored at a particular point in time (so that a list is
// stable until synchronization has completed). The hash table won't
// preserve a fast ordering, the hashes will likely cause.
