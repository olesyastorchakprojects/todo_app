use super::page::HasId;
use super::TodoId;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub struct Todo {
    #[schema(value_type = String)]
    pub id: TodoId,
    pub text: String,
    pub completed: bool,
    #[serde(default)]
    pub group: String,
}

impl HasId<TodoId> for Todo {
    fn id(&self) -> TodoId {
        self.id
    }
}

fn apply_if_changed<T: PartialEq + Clone>(field: &mut T, new: &Option<T>) {
    if let Some(value) = new {
        if *field != *value {
            *field = value.clone();
        }
    }
}

impl Todo {
    pub(crate) fn new(id: TodoId, text: &str) -> Self {
        Self {
            id,
            text: text.to_owned(),
            completed: false,
            group: String::new(),
        }
    }
    pub(crate) fn apply(&mut self, update: &UpdateTodo) {
        apply_if_changed(&mut self.text, &update.text);
        apply_if_changed(&mut self.completed, &update.completed);
        apply_if_changed(&mut self.group, &update.group);
    }
}
#[derive(Debug)]
pub struct UpdateTodo {
    pub text: Option<String>,
    pub completed: Option<bool>,
    pub group: Option<String>,
}

impl From<&crate::handlers::UpdateTodo> for UpdateTodo {
    fn from(value: &crate::handlers::UpdateTodo) -> Self {
        Self {
            text: value.text.clone(),
            completed: value.completed,
            group: value.group.clone(),
        }
    }
}

#[derive(Encode, Decode, Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
pub(crate) enum TodoVersion {
    V1 {
        id: TodoId,
        text: String,
        completed: bool,
    },
    V2 {
        id: TodoId,
        text: String,
        completed: bool,
        group: String,
    },
}

impl From<TodoVersion> for Todo {
    fn from(value: TodoVersion) -> Self {
        match value {
            TodoVersion::V1 {
                id,
                text,
                completed,
            } => Self {
                id,
                text,
                completed,
                group: String::default(),
            },
            TodoVersion::V2 {
                id,
                text,
                completed,
                group,
            } => Self {
                id,
                text,
                completed,
                group,
            },
        }
    }
}

impl From<Todo> for TodoVersion {
    fn from(value: Todo) -> Self {
        Self::V2 {
            id: value.id,
            text: value.text,
            completed: value.completed,
            group: value.group,
        }
    }
}
