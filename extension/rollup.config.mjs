import typescript from '@rollup/plugin-typescript';

export default [
  {
    input: 'src/background.ts',
    output: {
      file: 'dist/background.js',
      format: 'es'
    },
    plugins: [typescript()]
  },
  {
    input: 'src/content-script.ts',
    output: {
      file: 'dist/content-script.js',
      format: 'es'
    },
    plugins: [typescript()]
  }
];