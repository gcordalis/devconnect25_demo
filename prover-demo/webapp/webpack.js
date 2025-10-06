var webpack = require('webpack'),
  path = require('path'),
  CopyWebpackPlugin = require('copy-webpack-plugin'),
  HtmlWebpackPlugin = require('html-webpack-plugin');

const ASSET_PATH = process.env.ASSET_PATH || '/';

var alias = {};

var fileExtensions = [
  'jpg',
  'jpeg',
  'png',
  'gif',
  'svg',
  'eot',
  'otf',
  'svg',
  'ttf',
  'woff',
  'woff2',
];

var options = {
  ignoreWarnings: [
    /Circular dependency between chunks with runtime/,
    /ResizeObserver loop completed with undelivered notifications/,
  ],
  mode: 'development',
  entry: {
    app: path.join(__dirname, 'src', 'app.tsx'),
  },
  output: {
    filename: '[name].bundle.js',
    path: path.resolve(__dirname, 'build'),
    clean: true,
    publicPath: ASSET_PATH,
  },
  module: {
    rules: [
      {
        test: /\.svg$/,
        use: [
          {
            loader: '@svgr/webpack',
            options: {
              typescript: true,
              ext: 'tsx',
            },
          },
        ],
      },
      {
        test: new RegExp('.(' + fileExtensions.filter(ext => ext !== 'svg').join('|') + ')$'),
        type: 'asset/resource',
        exclude: /node_modules/,
      },
      {
        test: /\.html$/,
        loader: 'html-loader',
        exclude: /node_modules/,
      },
      {
        test: /\.(ts|tsx)$/,
        exclude: /node_modules/,
        use: [
          {
            loader: 'source-map-loader',
          },
          {
            loader: require.resolve('ts-loader'),
          },
        ],
      },
      {
        test: /\.(js|jsx)$/,
        use: [
          {
            loader: 'source-map-loader',
          },
          {
            loader: require.resolve('babel-loader'),
          },
        ],
        exclude: /node_modules/,
      },
      {
        // look for .css or .scss files
        test: /\.(css|scss)$/,
        // in the `web` directory
        use: [
          {
            loader: 'style-loader',
          },
          {
            loader: 'css-loader',
            options: { importLoaders: 1 },
          },
          {
            loader: 'postcss-loader',
          },
          {
            loader: 'sass-loader',
            options: {
              sourceMap: true,
            },
          },
        ],
      },
    ],
  },
  resolve: {
    alias: alias,
    extensions: fileExtensions
      .map((extension) => '.' + extension)
      .concat(['.js', '.jsx', '.ts', '.tsx', '.css', 'svg']),
    fallback: {
      crypto: require.resolve('crypto-browserify'),
      stream: require.resolve('stream-browserify'),
      vm: require.resolve('vm-browserify'),
    },
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        {
          from: 'tlsn-wasm-pkg/',
          to: path.join(__dirname, 'build'),
          force: true,
        },
      ],
    }),
    new CopyWebpackPlugin({
      patterns: [{ from: '../README.md', to: 'README.md' }],
    }),
    new HtmlWebpackPlugin({
      template: path.join(__dirname, 'index.ejs'),
      filename: 'index.html',
      cache: false,
    }),
    new webpack.ProvidePlugin({
      process: 'process/browser',
    }),
    new webpack.ProvidePlugin({
      Buffer: ['buffer', 'Buffer'],
    }),
    new webpack.DefinePlugin({
      'process.env.PROVER_PROXY_URL': JSON.stringify(process.env.PROVER_PROXY_URL || 'ws://localhost:9816/prove'),
    }),
  ].filter(Boolean),
  // Required by wasm-bindgen-rayon, in order to use SharedArrayBuffer on the Web
  // Ref:
  //  - https://github.com/GoogleChromeLabs/wasm-bindgen-rayon#setting-up
  //  - https://web.dev/i18n/en/coop-coep/
  devServer: {
    port: 8080,
    host: 'localhost',
    hot: true,
    headers: {
      'Cross-Origin-Embedder-Policy': 'require-corp',
      'Cross-Origin-Opener-Policy': 'same-origin',
    },
    client: {
      overlay: false,
    },
  },
};

module.exports = options;
