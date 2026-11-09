"use server";

import { revalidatePath } from "next/cache";
import { CELEBRATE_REACTION, getFeed, react } from "@/lib/duolingo/client";
import { canCelebrate } from "@/lib/duolingo/format";

/** Sends a "Celebrate" reaction for one update, then refreshes the page's data. */
export async function celebrateAction(eventId: string): Promise<void> {
  console.log(`[actions] celebrate: event_id=${eventId}`);
  await react(eventId, CELEBRATE_REACTION);
  revalidatePath("/feed");
}

/**
 * Celebrates every update that can still be celebrated with CONGRATS (see
 * format.ts::canCelebrate -- not just "has an event id": FOLLOW cards have one too, but
 * no reaction mechanism Duolingo tracks for them, and SHARE_SENTENCE_OFFER uses
 * like/haha/love, not congrats) and hasn't already been reacted to. Mirrors
 * rust/core/src/controller.rs::celebrate_all(). Keeps going even if some calls fail,
 * logging each one, then refreshes the page's data once at the end.
 */
export async function celebrateAllAction(): Promise<void> {
  const cards = await getFeed();
  const eventIds = cards
    .filter((card) => canCelebrate(card) && !card.reactionType)
    .map((card) => card.eventId as string);

  console.log(`[actions] celebrate_all: ${eventIds.length} eligible cards`);

  let succeeded = 0;
  let failed = 0;
  for (const eventId of eventIds) {
    try {
      await react(eventId, CELEBRATE_REACTION);
      succeeded++;
    } catch (e) {
      console.warn(`[actions] celebrate_all: event_id=${eventId} failed: ${e}`);
      failed++;
    }
  }

  console.log(`[actions] celebrate_all: done, ${succeeded} succeeded, ${failed} failed`);
  revalidatePath("/feed");
}
