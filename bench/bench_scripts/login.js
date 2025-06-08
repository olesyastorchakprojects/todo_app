import { randomItem } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';
import { users, tokens, login } from './common.js';

export const options = {
  systemTags: [],
  scenarios: {
    warm_up: {
      executor: 'constant-arrival-rate',
      rate: 40,
      timeUnit: '1s',
      duration: '30s',
      preAllocatedVUs: 50,
      maxVUs: users,
      exec: 'loginTest',
      startTime: '0s',
      tags: { phase: 'warmup' }
    },
    login: {
      executor: 'constant-arrival-rate',
      rate: 40,
      timeUnit: '1s',
      duration: '60s',
      preAllocatedVUs: 50,
      maxVUs: users,
      startTime: '30s',
      tags: { phase: 'bench' },
      exec: 'loginTest',
    },
  },
//   thresholds: {
//     'http_req_duration{endpoint:http://todo-bench-app:3401/auth/register}': ['p(95)<250'],
//     'http_req_duration{endpoint:http://todo-bench-app:3401/auth/login}':    ['p(95)<250'],
//   },
};

export function loginTest() {
  let token_data = randomItem(tokens);

  login(token_data.email, token_data.password);
}


