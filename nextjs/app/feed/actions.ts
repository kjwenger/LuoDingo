"use server";

import { revalidatePath } from "next/cache";
import { CELEBRATE_REACTION, getFeed, react } from "@/lib/duolingo/client";

/** Sends a "Celebrate" reaction for one update, then refreshes the page's data. */
export async function celebrateAction(eventId: string): Promise<void> {
  console.log(`[actions] celebrate: event_id=${eventId}`);
  await react(eventId, CELEBRATE_REACTION);
  revalidatePath("/feed");
}

/**
 * Celebrates every update that can still be celebrated (has an event id -- FOLLOW cards
 * don't -- and hasn't already been reacted to), mirroring
 * rust/core/src/controller.rs::celebrate_all(). Keeps going even if some calls fail,
 * logging each one, then refreshes the page's data once at the end.
 */
export async function celebrateAllAction(): Promise<void> {
  const cards = await getFeed();
  const eventIds = cards
    .filter((card) => card.eventId && !card.reactionType)
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
