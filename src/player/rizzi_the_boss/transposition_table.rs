use super::*;

pub const TRANSPOSITION_TABLE_SIZE_MB: usize = 64;
const SIZE_ENTRY: usize = size_of::<TableEntry>();
pub const NUM_ENTRIES: usize = 1024 * 1024 * TRANSPOSITION_TABLE_SIZE_MB / SIZE_ENTRY;

#[derive(Default, Debug, PartialEq)]
pub enum TypeNode {
    #[default]
    PV, // exact
    All, // lower bound
    Cut, // upper bound
}

#[derive(/*Default,*/ Debug)]
pub struct TableEntry {
    pub key: Hash,
    pub node: TypeNode,
    pub r#move: Move,
    pub depth: u16,
    pub score: Eval,
    // DEBUG
    //pub board: [Piece; 64]
    
}

impl Default for TableEntry {
    fn default() -> Self {
        TableEntry { key: Default::default(), node: Default::default(), r#move: Default::default(), depth: Default::default(), score: Default::default()/*, board: [NONE; 64]*/ }
    }
}

#[derive(Debug)]
pub struct TranspositionTable {
    table: Vec<TableEntry>,
    pub occupancy: u32,
    pub overwrites: u32,
    pub collisions: u32,
    //DEBUG
    /*pub hash_collision: u32,*/
}

impl TableEntry {
    pub fn new(hash: Hash, node: TypeNode, r#move: Move, depth: u16, eval: Eval/*, board: [Piece; 64]*/) -> Self {
        TableEntry { key: hash, node, r#move, depth, score: eval/*, board: board*/ }
    }
}

impl TranspositionTable {
    pub fn new() -> Self {
        let mut table = Vec::with_capacity(NUM_ENTRIES);
        table.resize_with(NUM_ENTRIES, Default::default);
        TranspositionTable { table, occupancy: 0, overwrites: 0, collisions: 0/*, hash_collision: 0*/ }
    }

    fn get_index(&self, hash: Hash) -> usize {hash as usize % NUM_ENTRIES}

    pub fn get_entry(&mut self, hash: Hash/*, board: &[Piece; 64]*/) -> Option<&TableEntry> {
        let entry = &self.table[self.get_index(hash)];
        if entry.key != hash {return None}
        //if entry.board.iter().ne(board.iter()) {self.hash_collision += 1}
        Some(entry)
    }

    pub fn put_entry(&mut self, entry: TableEntry) {
        assert!(entry.key != 0);
        let index = self.get_index(entry.key);
        if self.table[index].key == 0 {
            self.occupancy += 1;
        } else {
            self.overwrites += 1;
            if self.table[index].key != entry.key {
                self.collisions += 1;
            }
        }
        self.table[index] = entry;
    }
}