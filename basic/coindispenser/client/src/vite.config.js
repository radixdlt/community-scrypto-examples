import { viteStaticCopy } from 'vite-plugin-static-copy'

export default {
    root: './',
    build: {
        outDir: './dist',
    },
    plugins: [
        viteStaticCopy({
          targets: [
            {
              src: './.well-known',
              dest: './'
            }
          ]
        })
    ]
} 
    