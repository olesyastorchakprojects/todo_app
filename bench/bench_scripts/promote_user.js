import { randomItem } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';
import { users, tokens, login, promoteUser} from './common.js';

export const options = {
  systemTags: [],
  tags: { phase: 'warmup' },
  scenarios: {
    warm_up: {
      executor: 'constant-arrival-rate',
      rate: 10,
      timeUnit: '1s',
      duration: '30s',
      preAllocatedVUs: 50,
      maxVUs: users,
      exec: 'promoteUserTest',
      startTime: '0s',
      tags: { phase: 'warmup' }
    },
    promote_user: {
      executor: 'constant-arrival-rate',
      rate: 10,
      timeUnit: '1s',
      duration: '60s',
      preAllocatedVUs: 50,
      maxVUs: users,
      startTime: '30s',
      tags: { phase: 'bench' },
      exec: 'promoteUserTest',
    },
  },
//   thresholds: {
//     'http_req_duration{endpoint:http://todo-bench-app:3401/auth/register}': ['p(95)<250'],
//     'http_req_duration{endpoint:http://todo-bench-app:3401/auth/login}':    ['p(95)<250'],
//   },
};

export function setup() {
  return { admin_token: login("admin@gmail.com", "admin") };
}

export function promoteUserTest(data) {
  const token_data = randomItem(tokens);

  let admin_token = data.admin_token;

  promoteUser(admin_token, token_data.id);
}


