// force-dynamic: this page's data is a live, single-account read from Duolingo that
// should never be cached or prerendered -- see next.config.ts for why cacheComponents
// (which would otherwise require a <Suspense> boundary for this) is off.
export const dynamic = "force-dynamic";

import type { Metadata } from "next";
import { getFeed, getFollowers, getFollowing } from "@/lib/duolingo/client";
import type { FeedCard, Friend } from "@/lib/duolingo/types";
import { celebrateAction, celebrateAllAction } from "./actions";
import FeedView from "./FeedView";

export const metadata: Metadata = {
  title: "Friend Updates — LuoDingo",
};

function unwrap<T>(
  result: PromiseSettledResult<T>,
  label: string,
  errors: string[],
  fallback: T
): T {
  if (result.status === "fulfilled") return result.value;
  errors.push(`${label}: ${result.reason}`);
  return fallback;
}

export default async function FeedPage() {
  // Mirrors rust/core/src/controller.rs::refresh(): fetch all three in parallel, keep
  // whatever succeeded even if one fails, and surface a combined error message.
  const [feedResult, followingResult, followersResult] = await Promise.allSettled([
    getFeed(),
    getFollowing(),
    getFollowers(),
  ]);

  const errors: string[] = [];
  const updates = unwrap<FeedCard[]>(feedResult, "feed", errors, []);
  const following = unwrap<Friend[]>(followingResult, "following", errors, []);
  const followers = unwrap<Friend[]>(followersResult, "followers", errors, []);

  following.sort((a, b) => b.weeklyXp - a.weeklyXp);
  followers.sort((a, b) => b.weeklyXp - a.weeklyXp);

  return (
    <FeedView
      updates={updates}
      following={following}
      followers={followers}
      error={errors.length > 0 ? errors.join("; ") : null}
      celebrateAction={celebrateAction}
      celebrateAllAction={celebrateAllAction}
    />
  );
}
