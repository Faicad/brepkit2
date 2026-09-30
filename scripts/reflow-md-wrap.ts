/** One-off: reflow hard-wrapped prose paragraphs to one physical line per paragraph. */
import { readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { parseMarkdown } from './markdown.ts'

const root = resolve(import.meta.dirname, '..')
const files = process.argv.slice(2)

for (const f of files) {
  const abs = resolve(root, f)
  const src = readFileSync(abs, 'utf8')
  const lines = src.split('\n')
  const tree = parseMarkdown(src)
  // Collect paragraph line ranges (1-based), deepest-first to splice safely.
  const ranges: Array<{ start: number; end: number; prefix: string }> = []
  const visit = (node: any, prefix: string): void => {
    if (node.type === 'blockquote') {
      // Children render with the original "> " prefix preserved by line ranges.
      for (const child of node.children) visit(child, prefix)
      return
    }
    if (node.type === 'paragraph' && node.position) {
      ranges.push({ start: node.position.start.line, end: node.position.end.line, prefix })
    }
    for (const key of ['children']) {
      if (Array.isArray(node[key])) for (const c of node[key]) visit(c, prefix)
    }
  }
  visit(tree, '')
  ranges.sort((a, b) => b.start - a.start)
  for (const r of ranges) {
    if (r.end === r.start) continue
    const slice = lines.slice(r.start - 1, r.end)
    // Preserve per-line prefixes like "> " so a quoted paragraph stays quoted.
    const merged = slice
      .map((l, i) => (i === 0 ? l : l.replace(/^(\s*>\s?)/, '')))
      .join(' ')
      .replace(/\s+/g, ' ')
      .trim()
    lines.splice(r.start - 1, r.end - r.start + 1, merged)
  }
  writeFileSync(abs, lines.join('\n'))
  console.log(`reflowed ${f}: ${ranges.filter(r => r.end !== r.start).length} paragraph(s)`)
}
