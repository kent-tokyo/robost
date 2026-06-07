module.exports = {
  entry: {
    index: './src/main/index.ts',
    preload: './src/main/preload.ts',
  },
  output: {
    path: require('path').resolve(__dirname, '.webpack/main'),
    filename: '[name].js',
  },
  module: {
    rules: require('./webpack.rules'),
  },
  resolve: {
    extensions: ['.ts', '.js'],
  },
};
