FROM oven/bun:1 AS builder
WORKDIR /app

COPY package.json ./
COPY bun.lockb ./

RUN bun install

COPY . ./

RUN bun run build

################################################################################

FROM oven/bun:1 as runtime
WORKDIR /app

COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/bun.lockb ./bun.lockb

RUN bun install --production

RUN groupadd -r appgroup && useradd -r -g appgroup appuser
RUN chown -R appuser:appgroup /app

USER appuser

EXPOSE 3000

CMD ["bun", "start"]
