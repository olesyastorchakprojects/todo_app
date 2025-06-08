import { users, createTodo, getTodos, deleteTodo, updateTodo, deleteAllTodos } from './common.js';

export const options = {
  systemTags: [],
  scenarios: {
    warm_up: {
      executor: 'constant-arrival-rate',
      rate: 500,
      timeUnit: '1s',
      duration: '1m',
      preAllocatedVUs: 100,
      maxVUs: 500,
      exec: 'createTodoTest',
      startTime: '0s',
      tags: { phase: 'warmup' }
    },
    write_create: {
      executor: 'constant-arrival-rate',
      rate: 50,
      timeUnit: '1s',
      duration: '10m',
      preAllocatedVUs: 50,
      maxVUs: users,
      exec: 'createTodoTest',
      startTime: '1m',
      tags: { phase: 'bench' }
    },
    write_delete: {
      executor: 'constant-arrival-rate',
      rate: 50,
      timeUnit: '1s',
      duration: '10m',
      preAllocatedVUs: 50,
      maxVUs: users,
      exec: 'deleteTodoTest',
      startTime: '1m',
      tags: { phase: 'bench' }
    },
    read_get_all: {
      executor: 'constant-arrival-rate',
      rate: 50,
      timeUnit: '1s',
      duration: '10m',
      preAllocatedVUs: 50,
      maxVUs: users,
      exec: 'getTodosTest',
      startTime: '1m',
      tags: { phase: 'bench' }
    },
    write_update: {
      executor: 'constant-arrival-rate',
      rate: 50,
      timeUnit: '1s',
      duration: '10m',
      preAllocatedVUs: 50,
      maxVUs: users,
      exec: 'updateTodoTest',
      startTime: '1m',
      tags: { phase: 'bench' }
    },
    write_delete_all: {
      executor: 'constant-arrival-rate',
      rate: 50,
      timeUnit: '1s',
      duration: '10m',
      preAllocatedVUs: 50,
      maxVUs: users,
      exec: 'deleteAllTodosTest',
      startTime: '1m',
      tags: { phase: 'bench' }
    },
  },
};

export function createTodoTest() {
  createTodo();
}

export function getTodosTest() {
  getTodos();
}

export function deleteTodoTest() {
  deleteTodo();
}

export function updateTodoTest() {
  updateTodo();
}

export function deleteAllTodosTest() {
  deleteAllTodos();
}
