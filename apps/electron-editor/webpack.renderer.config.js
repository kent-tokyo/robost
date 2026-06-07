const rules = require('./webpack.rules');

rules.push({
  test: /\.css$/,
  use: [{ loader: 'style-loader' }, { loader: 'css-loader' }],
});

module.exports = {
  entry: './src/renderer/index.tsx',
  module: {
    rules,
  },
  resolve: {
    extensions: ['.ts', '.tsx', '.js', '.jsx'],
    alias: {
      '@components': require('path').resolve(__dirname, 'src/renderer/components'),
      '@hooks': require('path').resolve(__dirname, 'src/renderer/hooks'),
      '@store': require('path').resolve(__dirname, 'src/renderer/store'),
      '@types': require('path').resolve(__dirname, 'src/renderer/types'),
      '@utils': require('path').resolve(__dirname, 'src/renderer/utils'),
    },
  },
};
