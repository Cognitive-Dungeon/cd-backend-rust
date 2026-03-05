import svelte from 'rollup-plugin-svelte';
import replace from "@rollup/plugin-replace";
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import postcss from 'rollup-plugin-postcss';
import { terser } from 'rollup-plugin-terser';

const production = !process.env.ROLLUP_WATCH;

export default {
    input: 'src/main.js',
    output: {
        sourcemap: true,
        format: 'iife',
        name: 'app',
        file: 'out/compiled/bundle.js'
    },
    plugins: [
        postcss({ plugins: [] }),
        svelte({
            dev: !production,
            css: css => { css.write('bundle.css'); }
        }),
        replace({
            "process.env.NODE_ENV": JSON.stringify(production ? "production" : "development"),
            preventAssignment: true,
        }),
        resolve({
            browser: true,
            dedupe: ['svelte', 'sveltejs-tippy']
        }),
        commonjs(),
        production && terser()
    ]
};