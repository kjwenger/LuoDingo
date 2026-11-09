"use client";

import { useRouter } from "next/navigation";
import { type ReactNode, useState, useTransition } from "react";
import {
  avatarUrl,
  badge,
  commentIndicator,
  humanizeTimestamp,
  reactionsSummary,
} from "@/lib/duolingo/format";
import type { FeedCard, Friend } from "@/lib/duolingo/types";

// Same approximate Duolingo dark-theme palette as rust/egui-app/src/main.rs, for visual
// parity between the two frontends.
const BG = "#131F24";
const CARD_BG = "#1F2D35";
const ACCENT_GREEN = "#58CC02";
const MUTED = "#8A8A8A";

type Tab = "updates" | "following" | "followers";

interface Props {
  updates: FeedCard[];
  following: Friend[];
  followers: Friend[];
  error: string | null;
  celebrateAction: (eventId: string) => Promise<void>;
  celebrateAllAction: () => Promise<void>;
}

export default function FeedView({
  updates,
  following,
  followers,
  error,
  celebrateAction,
  celebrateAllAction,
}: Props) {
  const router = useRouter();
  const [tab, setTab] = useState<Tab>("updates");
  const [isPending, startTransition] = useTransition();

  const celebratableCount = updates.filter((c) => c.eventId && !c.reactionType).length;

  return (
    <div style={{ background: BG, color: "#fff", minHeight: "100vh" }} className="p-4 font-sans">
      <div className="mb-3 flex items-center gap-3">
        <h1 className="text-xl font-bold" style={{ color: ACCENT_GREEN }}>
          LuoDingo
        </h1>
        <button
          onClick={() => router.refresh()}
          disabled={isPending}
          className="rounded bg-neutral-700 px-3 py-1 text-sm hover:bg-neutral-600 disabled:opacity-50"
        >
          Refresh
        </button>
        <button
          onClick={() => startTransition(() => celebrateAllAction())}
          disabled={isPending || celebratableCount === 0}
          className="rounded bg-neutral-700 px-3 py-1 text-sm hover:bg-neutral-600 disabled:opacity-50"
        >
          {"\u{1F389}"} Celebrate All ({celebratableCount})
        </button>
      </div>

      {error && <p className="mb-3 text-sm text-red-400">{error}</p>}

      <div className="mb-4 flex gap-4 border-b border-neutral-700 text-sm">
        <TabButton active={tab === "updates"} onClick={() => setTab("updates")}>
          Updates ({updates.length})
        </TabButton>
        <TabButton active={tab === "following"} onClick={() => setTab("following")}>
          Following ({following.length})
        </TabButton>
        <TabButton active={tab === "followers"} onClick={() => setTab("followers")}>
          Followers ({followers.length})
        </TabButton>
      </div>

      {tab === "updates" && (
        <UpdatesList
          cards={updates}
          pending={isPending}
          onCelebrate={(eventId) => startTransition(() => celebrateAction(eventId))}
        />
      )}
      {tab === "following" && <FriendsTable friends={following} />}
      {tab === "followers" && <FriendsTable friends={followers} />}
    </div>
  );
}

function TabButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className="-mb-px border-b-2 pb-2"
      style={{
        borderColor: active ? ACCENT_GREEN : "transparent",
        color: active ? "#fff" : MUTED,
      }}
    >
      {children}
    </button>
  );
}

function Avatar({ url, size }: { url: string | null; size: number }) {
  if (!url) {
    return (
      <div
        className="flex-shrink-0 rounded-full"
        style={{ background: MUTED, width: size, height: size }}
      />
    );
  }
  // Plain <img>, same reasoning as the Rust frontends: Duolingo's avatar hosts vary
  // (render CDN vs. default-avatar host) and both need the /medium size-suffix trick
  // (see avatarUrl()), which doesn't play well with next/image's own remote-pattern
  // allowlist + optimization query params.
  return (
    // eslint-disable-next-line @next/next/no-img-element
    <img
      src={avatarUrl(url)}
      alt=""
      className="flex-shrink-0 rounded-full"
      style={{ width: size, height: size }}
    />
  );
}

function UpdatesList({
  cards,
  pending,
  onCelebrate,
}: {
  cards: FeedCard[];
  pending: boolean;
  onCelebrate: (eventId: string) => void;
}) {
  if (cards.length === 0) {
    return <p style={{ color: MUTED }}>No recent friend updates.</p>;
  }

  return (
    <div className="flex flex-col gap-2">
      {cards.map((card) => {
        const b = badge(card);
        const comments = commentIndicator(card);
        const reactions = reactionsSummary(card);
        const already = Boolean(card.reactionType);

        return (
          <div
            key={card.eventId ?? `${card.userId}-${card.timestamp}`}
            className="flex gap-3 rounded-lg p-3"
            style={{ background: CARD_BG }}
          >
            <Avatar url={card.picture} size={44} />

            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <span className="font-bold">{card.displayName ?? "Someone"}</span>
                <span className="text-xs" style={{ color: MUTED }}>
                  {humanizeTimestamp(card.timestamp)}
                </span>
                {b && (
                  <span
                    className="ml-auto flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full text-xs font-bold text-black"
                    style={{ background: `rgb(${b.color.r}, ${b.color.g}, ${b.color.b})` }}
                  >
                    {b.glyph}
                  </span>
                )}
              </div>

              <p className="text-sm">{card.body || card.cardType}</p>

              <div className="mt-1 flex items-center gap-3 text-xs">
                {card.eventId && (
                  <button
                    onClick={() => onCelebrate(card.eventId as string)}
                    disabled={pending || already}
                    className="rounded bg-neutral-700 px-2 py-1 hover:bg-neutral-600 disabled:opacity-50"
                  >
                    {already ? `${"\u{1F389}"} Celebrated` : `${"\u{1F389}"} Celebrate`}
                  </button>
                )}
                {reactions && <span style={{ color: MUTED }}>{reactions}</span>}
                {comments && <span style={{ color: MUTED }}>{comments}</span>}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

function FriendsTable({ friends }: { friends: Friend[] }) {
  if (friends.length === 0) {
    return <p style={{ color: MUTED }}>Nobody here.</p>;
  }

  return (
    <table className="w-full text-sm">
      <thead>
        <tr className="text-left" style={{ color: MUTED }}>
          <th className="py-1"></th>
          <th className="py-1">Name</th>
          <th className="py-1">Username</th>
          <th className="py-1">Total XP</th>
          <th className="py-1">Weekly XP</th>
          <th className="py-1">Monthly XP</th>
          <th className="py-1">Active</th>
        </tr>
      </thead>
      <tbody>
        {friends.map((friend) => (
          <tr key={friend.id} className="border-t border-neutral-800">
            <td className="py-1">
              <Avatar url={friend.picture} size={28} />
            </td>
            <td className="py-1">
              {friend.name ?? friend.username}
              {friend.hasPlus ? " \u{1F48E}" : ""}
            </td>
            <td className="py-1">{friend.username}</td>
            <td className="py-1">{friend.totalXp}</td>
            <td className="py-1">{friend.weeklyXp}</td>
            <td className="py-1">{friend.monthlyXp}</td>
            <td className="py-1">{friend.hasRecentActivity15 ? "\u{1F7E2}" : ""}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
