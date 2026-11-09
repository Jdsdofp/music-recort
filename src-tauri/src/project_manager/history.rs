pub struct UndoRedoHistory<T: Clone> {
    states: Vec<T>,
    cursor: usize,
    max_size: usize,
}

impl<T: Clone> UndoRedoHistory<T> {
    pub fn new(initial: T) -> Self {
        UndoRedoHistory {
            states: vec![initial],
            cursor: 0,
            max_size: 100,
        }
    }

    pub fn push(&mut self, state: T) {
        // Descarta estados futuros (redo branch)
        self.states.truncate(self.cursor + 1);
        self.states.push(state);
        // Se exceder o limite, remove o mais antigo e mantém cursor correto
        if self.states.len() > self.max_size + 1 {
            self.states.remove(0);
            // cursor permanece em self.states.len() - 1 após remoção
            self.cursor = self.states.len() - 1;
        } else {
            self.cursor = self.states.len() - 1;
        }
    }

    pub fn undo(&mut self) -> Option<T> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor -= 1;
        Some(self.states[self.cursor].clone())
    }

    pub fn redo(&mut self) -> Option<T> {
        if self.cursor >= self.states.len() - 1 {
            return None;
        }
        self.cursor += 1;
        Some(self.states[self.cursor].clone())
    }

    pub fn current(&self) -> &T {
        &self.states[self.cursor]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo_basic() {
        let mut h = UndoRedoHistory::new(0i32);
        h.push(1);
        h.push(2);
        assert_eq!(*h.current(), 2);
        assert_eq!(h.undo(), Some(1));
        assert_eq!(h.undo(), Some(0));
        assert_eq!(h.undo(), None);
        assert_eq!(h.redo(), Some(1));
    }

    #[test]
    fn test_push_after_undo_clears_redo() {
        let mut h = UndoRedoHistory::new(0i32);
        h.push(1);
        h.push(2);
        h.undo();
        h.push(99);
        assert_eq!(h.redo(), None);
        assert_eq!(*h.current(), 99);
    }
}
