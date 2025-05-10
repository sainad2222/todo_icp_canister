use candid::{CandidType, Deserialize};
use ic_cdk::{query, update};
use std::cell::{Cell, RefCell};

#[derive(CandidType, Deserialize, Clone)]
struct Todo {
    id: u64,
    text: String,
    deleted: bool, // soft delete flag
}

thread_local! {
    static TODOS: RefCell<Vec<Todo>> = RefCell::new(Vec::new());
    static NEXT_ID: Cell<u64> = Cell::new(1);
}

/// Add a new todo → returns generated id
#[update]
fn add_todo(text: String) -> u64 {
    let id = NEXT_ID.with(|n| {
        let current = n.get();
        n.set(current + 1);
        current
    });
    TODOS.with(|todos| {
        todos.borrow_mut().push(Todo {
            id,
            text,
            deleted: false,
        });
    });
    id
}

/// Get a todo by id → returns Option<Todo> (None if deleted or not found)
#[query]
fn get_todo(id: u64) -> Option<Todo> {
    TODOS.with(|todos| {
        todos
            .borrow()
            .iter()
            .find(|todo| todo.id == id && !todo.deleted)
            .cloned()
    })
}

/// List todos (paginated) → start after `after_id`, return up to `limit`
#[query]
fn list_todos(after_id: Option<u64>, limit: usize) -> Vec<Todo> {
    TODOS.with(|todos| {
        let todos = todos.borrow();

        let iter: Box<dyn Iterator<Item = &Todo>> = if let Some(id) = after_id {
            Box::new(
                todos
                    .iter()
                    .filter(|todo| !todo.deleted)
                    .skip_while(move |todo| todo.id != id)
                    .skip(1),
            )
        } else {
            Box::new(todos.iter().filter(|todo| !todo.deleted))
        };

        iter.take(limit).cloned().collect()
    })
}

/// Update a todo’s text by id → returns Option<Todo>
#[update]
fn update_todo(id: u64, new_text: String) -> Option<Todo> {
    TODOS.with(|todos| {
        let mut todos = todos.borrow_mut();
        if let Some(todo) = todos.iter_mut().find(|todo| todo.id == id && !todo.deleted) {
            todo.text = new_text;
            Some(todo.clone())
        } else {
            None
        }
    })
}

/// Soft delete a todo by id → marks deleted=true → returns Option<Todo>
#[update]
fn delete_todo(id: u64) -> Option<Todo> {
    TODOS.with(|todos| {
        let mut todos = todos.borrow_mut();
        if let Some(todo) = todos.iter_mut().find(|todo| todo.id == id && !todo.deleted) {
            todo.deleted = true;
            Some(todo.clone())
        } else {
            None
        }
    })
}

ic_cdk::export_candid!();
