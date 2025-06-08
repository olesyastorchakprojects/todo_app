import { randomEmail, register } from './common.js';

export const options = {
  systemTags: [],
  scenarios: {
    warm_up: {
      executor: 'constant-arrival-rate',
      rate: 30,
      timeUnit: '1s',
      duration: '1m',
      preAllocatedVUs: 50,
      maxVUs: 100,
      exec: 'registerTest',
      startTime: '0s',
      tags: { phase: 'warmup' }
    },
    registration: {
      executor: 'constant-arrival-rate',
      rate: 30,
      timeUnit: '1s',
      duration: '2m',
      preAllocatedVUs: 50,
      maxVUs: 100,
      exec: 'registerTest',
      startTime: '1m',
      tags: { phase: 'bench' }
    },
  },
//   thresholds: {
//     'http_req_duration{endpoint:http://todo-bench-app:3401/auth/register}': ['p(95)<250'],
//     'http_req_duration{endpoint:http://todo-bench-app:3401/auth/login}':    ['p(95)<250'],
//   },
};

export function registerTest() {
  const email = randomEmail();
  register(email);
}

