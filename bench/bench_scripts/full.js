import { users, createTodo, getTodos, deleteTodo, updateTodo, login, register, promoteUser, deleteAllTodos, randomEmail, tokens } from './common.js';
import { randomItem } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

export const options = {
  systemTags: [],
  tags: { phase: 'warmup' },
  scenarios: {
    warm_up: {
      executor: 'constant-arrival-rate',
      rate: 500,
      timeUnit: '1s',
      duration: '2m',
      preAllocatedVUs: 100,
      maxVUs: users,
      exec: 'createTodoTest',
      startTime: '0s',
      tags: { phase: 'warmup' }
    },
    warm_up_reg: {
      executor: 'constant-arrival-rate',
      rate: 20,
      timeUnit: '1s',
      duration: '2m',
      preAllocatedVUs: 10,
      maxVUs: users,
      exec: 'registerTest',
      startTime: '0s',
      tags: { phase: 'warmup' }
    },
    warm_up_update_todo: {
      executor: 'constant-arrival-rate',
      rate: 30,
      timeUnit: '1s',
      duration: '2m',
      preAllocatedVUs: 30,
      maxVUs: users,
      exec: 'updateTodoTest',
      startTime: '0s',
      tags: { phase: 'warmup' }
    },
    warm_up_delete_all_todos: {
      executor: 'constant-arrival-rate',
      rate: 30,
      timeUnit: '1s',
      duration: '2m',
      preAllocatedVUs: 30,
      maxVUs: users,
      exec: 'deleteAllTodosTest',
      startTime: '0s',
      tags: { phase: 'warmup' }
    },
    registration: {
      executor: 'constant-arrival-rate',
      rate: 8,
      timeUnit: '1s',
      duration: '20m',
      preAllocatedVUs: 10,
      maxVUs: users,
      exec: 'registerTest',
      startTime: '2m',
      tags: { phase: 'bench' }
    },
    login: {
        executor: 'constant-arrival-rate',
        rate: 10,
        timeUnit: '1s',
        duration: '20m',
        preAllocatedVUs: 10,
        maxVUs: users,
        startTime: '2m',
        tags: { phase: 'bench' },
        exec: 'loginTest',
    },
    promote_user: {
      executor: 'constant-arrival-rate',
      rate: 10,
      timeUnit: '1s',
      duration: '20m',
      preAllocatedVUs: 10,
      maxVUs: users,
      startTime: '2m',
      tags: { phase: 'bench' },
      exec: 'promoteUserTest',
    },
    create_todo: {
      executor: 'constant-arrival-rate',
      rate: 50,
      timeUnit: '1s',
      duration: '20m',
      preAllocatedVUs: 50,
      maxVUs: users,
      exec: 'createTodoTest',
      startTime: '2m',
      tags: { phase: 'bench' }
    },
    delete_todo_with_fallback: {
      executor: 'constant-arrival-rate',
      rate: 50,
      timeUnit: '1s',
      duration: '20m',
      preAllocatedVUs: 50,
      maxVUs: users,
      exec: 'deleteTodoTest',
      startTime: '2m',
      tags: { phase: 'bench' }
    },
    get_all_todos: {
      executor: 'constant-arrival-rate',
      rate: 50,
      timeUnit: '1s',
      duration: '20m',
      preAllocatedVUs: 50,
      maxVUs: users,
      exec: 'getTodosTest',
      startTime: '2m',
      tags: { phase: 'bench' }
    },
    update_todo: {
      executor: 'constant-arrival-rate',
      rate: 30,
      timeUnit: '1s',
      duration: '20m',
      preAllocatedVUs: 30,
      maxVUs: users,
      exec: 'updateTodoTest',
      startTime: '2m',
      tags: { phase: 'bench' }
    },
    delete_all_todos: {
      executor: 'constant-arrival-rate',
      rate: 30,
      timeUnit: '1s',
      duration: '20m',
      preAllocatedVUs: 30,
      maxVUs: users,
      exec: 'deleteAllTodosTest',
      startTime: '2m',
      tags: { phase: 'bench' }
    },
  },
};

export function setup() {
  return { admin_token: login("admin@gmail.com", "admin") };
}

export function registerTest() {
  const email = randomEmail();
  register(email);
}

export function loginTest() {
  let token_data = randomItem(tokens);

  login(token_data.email, token_data.password);
}

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

export function promoteUserTest(data) {
  const token_data = randomItem(tokens);

  let admin_token = data.admin_token;

  promoteUser(admin_token, token_data.id);
}

export function deleteAllTodosTest() {
  deleteAllTodos();
}
