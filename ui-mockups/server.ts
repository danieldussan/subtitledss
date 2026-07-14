const server = Bun.serve({
  port: 3456,
  async fetch(req) {
    const url = new URL(req.url);
    let path = url.pathname;
    if (path === "/") path = "/index.html";
    const file = Bun.file(`.${path}`);
    if (await file.exists()) {
      return new Response(file);
    }
    return new Response("Not Found", { status: 404 });
  },
});
console.log(`Mockups running at http://localhost:${server.port}`);
