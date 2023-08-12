import fs from 'fs';
import pool from '../pool.json' assert { type: 'json' };

const randomisedPool = {
  users: pool.users.map((user) => {
    return {
      ...user,
      is_mod: Math.random() < 0.03,
      is_vip: Math.random() < 0.06,
      is_sub: Math.random() < 0.2,
    };
  }),
};

const poolString = JSON.stringify(randomisedPool, null, 2);

fs.writeFileSync('pool.json', poolString);
