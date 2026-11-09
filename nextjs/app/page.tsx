export default function Home() {
  return (
    <div className="flex flex-1 items-center justify-center bg-zinc-50 font-sans dark:bg-black">
      <main className="flex w-full max-w-xl flex-col items-start gap-6 px-8 py-24">
        <h1 className="text-3xl font-semibold tracking-tight text-black dark:text-zinc-50">
          LuoDingo &mdash; Duolingo API (Unofficial)
        </h1>
        <p className="text-lg leading-8 text-zinc-600 dark:text-zinc-400">
          A reverse-engineered OpenAPI description of the Duolingo endpoints used by{" "}
          <a
            className="font-medium text-zinc-950 underline dark:text-zinc-50"
            href="https://github.com/kjwenger/LuoDingo"
          >
            LuoDingo
          </a>
          , plus a web UI with equivalent functionality to its Rust desktop apps.
        </p>
        <div className="flex gap-3">
          <a
            className="flex h-12 items-center justify-center rounded-full bg-foreground px-6 text-background transition-colors hover:bg-[#383838] dark:hover:bg-[#ccc]"
            href="/feed"
          >
            Open Friend Updates
          </a>
          <a
            className="flex h-12 items-center justify-center rounded-full border border-black/10 px-6 transition-colors hover:bg-black/5 dark:border-white/20 dark:hover:bg-white/10"
            href="/docs"
          >
            View API docs
          </a>
        </div>
      </main>
    </div>
  );
}
