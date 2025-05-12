
use candid::{CandidType, Deserialize};
use ic_cdk::{query, update};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct Todo {
    id: u64,
    text: String,
    deleted: bool, // soft delete flag
}

thread_local! {
    static TODOS: RefCell<HashMap<u64, Todo>> = RefCell::new(Default::default());
    static NEXT_ID: Cell<u64> = Cell::new(1);
}

// ===== Pure Logic (Testable) =====

fn insert_todo(map: &mut HashMap<u64, Todo>, id: u64, text: String) {
    map.insert(id, Todo { id, text, deleted: false });
}

fn get_visible_todo(map: &HashMap<u64, Todo>, id: u64) -> Option<Todo> {
    map.get(&id).cloned().filter(|t| !t.deleted)
}

fn list_visible_todos(map: &HashMap<u64, Todo>, after_id: Option<u64>, limit: usize) -> Vec<Todo> {
    let mut all: Vec<_> = map.values().filter(|t| !t.deleted).cloned().collect();
    all.sort_by_key(|t| t.id);
    if let Some(id) = after_id {
        if let Some(pos) = all.iter().position(|t| t.id == id) {
            all = all.into_iter().skip(pos + 1).collect();
        }
    }
    all.into_iter().take(limit).collect()
}

fn update_todo_text(map: &mut HashMap<u64, Todo>, id: u64, new_text: String) -> bool {
    if let Some(todo) = map.get_mut(&id) {
        if !todo.deleted {
            todo.text = new_text;
            return true;
        }
    }
    false
}

fn mark_todo_deleted(map: &mut HashMap<u64, Todo>, id: u64) -> bool {
    if let Some(todo) = map.get_mut(&id) {
        todo.deleted = true;
        return true;
    }
    false
}

// ===== Canister Interface =====

#[update]
fn add_todo(text: String) -> u64 {
    let id = NEXT_ID.with(|n| {
        let current = n.get();
        n.set(current + 1);
        current
    });
    TODOS.with(|todos| {
        insert_todo(&mut todos.borrow_mut(), id, text);
    });
    id
}

#[query]
fn get_todo(id: u64) -> Option<Todo> {
    TODOS.with(|todos| get_visible_todo(&todos.borrow(), id))
}

#[query]
fn list_todos(after_id: Option<u64>, limit: usize) -> Vec<Todo> {
    TODOS.with(|todos| list_visible_todos(&todos.borrow(), after_id, limit))
}

#[update]
fn update_todo(id: u64, new_text: String) -> bool {
    TODOS.with(|todos| update_todo_text(&mut todos.borrow_mut(), id, new_text))
}

#[update]
fn delete_todo(id: u64) -> bool {
    TODOS.with(|todos| mark_todo_deleted(&mut todos.borrow_mut(), id))
}

ic_cdk::export_candid!();

// ===== Unit Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut map = HashMap::new();
        insert_todo(&mut map, 1, "hello".into());
        let todo = get_visible_todo(&map, 1).unwrap();
        assert_eq!(todo.text, "hello");
        assert!(!todo.deleted);
    }

    #[test]
    fn test_soft_delete() {
        let mut map = HashMap::new();
        insert_todo(&mut map, 2, "delete me".into());
        assert!(mark_todo_deleted(&mut map, 2));
        assert!(get_visible_todo(&map, 2).is_none());
    }

    #[test]
    fn test_update_text() {
        let mut map = HashMap::new();
        insert_todo(&mut map, 3, "initial".into());
        assert!(update_todo_text(&mut map, 3, "updated".into()));
        assert_eq!(map.get(&3).unwrap().text, "updated");
    }

    #[test]
    fn test_list_with_pagination() {
        let mut map = HashMap::new();
        for id in 1..=5 {
            insert_todo(&mut map, id, format!("todo {id}"));
        }

        let first_2 = list_visible_todos(&map, None, 2);
        assert_eq!(first_2.len(), 2);
        assert_eq!(first_2[0].id, 1);
        assert_eq!(first_2[1].id, 2);

        let after_2 = list_visible_todos(&map, Some(2), 2);
        assert_eq!(after_2[0].id, 3);
    }
}
