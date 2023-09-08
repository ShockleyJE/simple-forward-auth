import http from 'k6/http';
import { sleep } from 'k6';

export let options = {
  stages: [
    { duration: '1m', target: 50 }, // Ramp up to 50 virtual users in 1 minute
    { duration: '5m', target: 50 }, // Stay at 50 virtual users for 5 minutes
    { duration: '1m', target: 0 },  // Ramp down to 0 virtual users in 1 minute
  ],
};

export default function () {
  let url = 'http://localhost:9090/';
  let headers = {
    'X-API-KEY': '7dd85a19-8893-482a-a6fa-e1a92f1c9d29'
  };

  http.get(url, { headers: headers });
  sleep(1); // Sleep for 1 second between requests
}