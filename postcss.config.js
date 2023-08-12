import presetEnv from 'postcss-preset-env';

/** @type {import('postcss-load-config').Config} */
const config = {
  plugins: [presetEnv()],
};

export default config;
