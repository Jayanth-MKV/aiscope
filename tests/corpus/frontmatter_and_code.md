---
applyTo: "**/*.ts"
---

# TypeScript rules

- Always use `async`/`await` over `.then()` promise chains.
- Prefer `pnpm` for installing dependencies.
- Co-locate tests as `*.test.ts` beside the file under test.

```ts
// camelCase example — this code block must be ignored by the parser
const userName = "alice";
```

> Quote-block content is also ignored.

Standalone paragraph: enable strict TypeScript.
