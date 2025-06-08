
import http from 'k6/http';
import { check } from 'k6';
import { SharedArray } from 'k6/data';
import { randomItem } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

export const BASE_URL = checkEnv("ENV_SERVER_URL");
export const TOKEN_FILE = checkEnv("ENV_TOKEN_FILE");
export const RANDOM_IP_COUNT = parseInt(checkEnv("RANDOM_IP_COUNT"));
export const TOKEN_COUNT = parseInt(checkEnv("ENV_TOKEN_COUNT"));
export const users = Number(TOKEN_COUNT);
if (isNaN(users) || users <= 0) {
  throw new Error("ENV_TOKEN_COUNT must be number");
}

export const tokens = new SharedArray('tokens', () => JSON.parse(open(TOKEN_FILE)));

export function checkEnv(name) {
  const val = __ENV[name];
  if (!val) throw new Error(`${name} is not set`);
  return val;
}

export function getAuthHeaders() {
  const ip = randomIp();
  const token = randomItem(tokens).token;
  return {
    Authorization: `Bearer ${token}`,
    'Content-Type': 'application/json',
    'X-Forwarded-For': ip
  };
}

function randomInt(min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

export function randomEmail() {
  const id = Math.floor(Math.random() * 1e12);
  return `bench_auth_testuser_${id}@example.com`;
}

function randomIp() {
  if (RANDOM_IP_COUNT === 0) {
      return Array.from({ length: 4 }, () => Math.floor(Math.random() * 256)).join('.');
  } else {
    const random_number = Math.floor(Math.random() * RANDOM_IP_COUNT);
    return `192.168.0.${random_number}`;
  }
}

export function register(email) {
  const ip = randomIp();
  const res = http.post(`${BASE_URL}/auth/register`, JSON.stringify({
    email: email,
    password: 'password123'
  }), {
    headers: { 'Content-Type': 'application/json', 'X-Forwarded-For': ip },
  });

  if (res.status === 409 ) {
    console.log(`409: ${email}`);
  }
  check(res, {
    'register 201 or 409': (r) => r.status === 201 || r.status === 409,
  });

  return res.status;
}

export function login(email, password) {
  const ip = randomIp();
  const res = http.post(`${BASE_URL}/auth/login`, JSON.stringify({
    email: email,
    password: password
  }), {
    headers: { 'Content-Type': 'application/json', 'X-Forwarded-For': ip },
  });

  const ok = check(res, {
    'login 200': (r) => r.status === 200,
    'login token exists': (r) => !!r.json('access_token'),
  });

  if (!ok) {
    return null;
  }
  const body = res.json();
  return body.access_token;
}

export function createTodo() {
  const res = http.post(`${BASE_URL}/todos`, JSON.stringify({ text: 'loadgen task' }), {
    headers: getAuthHeaders(),
  });
  check(res, { 'created': (r) => r.status === 201 });
}

export function getTodos() {
  const headers = getAuthHeaders();
  let cursor = null;
  let page = 0;

  do {
    const url = cursor
      ? `${BASE_URL}/todos?limit=10&after=${cursor}`
      : `${BASE_URL}/todos?limit=10`;

    const res = http.get(url, { headers });

    const ok = check(res, {'get_todos': (r) => r.status === 200});

    if (!ok) { return; }

    try {
      const body = res.json();
      cursor = (body && body.cursor) || null;
    } catch (e) {
      console.error(`Failed to parse page json ${page}:`, e.message);
      break;
    }

    page++;
  } while (cursor !== null && page < 2);
}

export function deleteTodo() {
  const headers = getAuthHeaders();

  const getRes = http.get(`${BASE_URL}/todos?limit=10`, { headers });
  const ok = check(getRes, {'get_todo': (r) => r.status === 200});

  if (!ok) { return; }

  const json = getRes.json();
  const id = json && json.items && json.items[0] && json.items[0].id;

  if (id) {
    const delRes = http.del(`${BASE_URL}/todos/${id}`, null, { headers });

    check(delRes, { 'deleted': (r) => r.status === 200 || r.status === 204 });
  } else {
    const createRes = http.post(`${BASE_URL}/todos`, JSON.stringify({ text: 'auto-generated task' }), {
      headers,
    });
    const ok = check(createRes, { 'created': (r) => r.status === 201 });
    if (!ok) { return; }
    const newId = createRes.json();
    if (newId) {
      const delRes = http.del(`${BASE_URL}/todos/${newId}`, null, { headers });
      check(delRes, { 'fallback deleted': (r) => r.status === 200 || r.status === 204});
    } else {
      console.warn("Failed to create fallback todo");
    }
  }
}

export function deleteAllTodos() {
  const headers = getAuthHeaders();

  const delRes = http.del(`${BASE_URL}/todos`, null, { headers });
  check(delRes, {'delete_all_todos': (r) => r.status === 200});
}

export function updateTodo() {
  const headers = getAuthHeaders();

  const getRes = http.get(`${BASE_URL}/todos?limit=10`, { headers });
  const ok = check(getRes, {'get_todo': (r) => r.status === 200});

  if (!ok) { return; }

  const json = getRes.json();
  const id = json && json.items && json.items[0] && json.items[0].id;

  if (id) {
    const updateRes = http.patch(`${BASE_URL}/todos/${id}`, JSON.stringify({ completed: true }), { headers });

    check(updateRes, { 'updated': (r) => r.status === 200 || r.status === 204 });
  } else {
    const createRes = http.post(`${BASE_URL}/todos`, JSON.stringify({ text: 'auto-generated task' }), {
        headers,
    });

    const ok = check(createRes, { 'created': (r) => r.status === 201 });
    if (!ok) { return; }

    const newId = createRes.json();
    if (newId) {

        const updateRes = http.patch(`${BASE_URL}/todos/${id}`, JSON.stringify({ completed: true }), { headers });

        check(updateRes, { 'fallback deleted': (r) => r.status === 200 || r.status === 204});

    } else {
        console.warn("Failed to create fallback todo");
    }
  }
}

export function promoteUser(admin_token, user_id) {
  if (admin_token === null){
    return;
  }

  const headers = {
    Authorization: `Bearer ${admin_token}`,
    'Content-Type': 'application/json',
  };

  const promoteRes = http.patch(`${BASE_URL}/admin/user/${user_id}/role`, JSON.stringify({ role: "admin" }), { headers });
  check(promoteRes, {'promote_user': (r) => r.status === 200});
}