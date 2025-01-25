import { type NextRequest, NextResponse } from "next/server";
import { env } from "~/env";
import { getSessionToken } from "~/lib/auth";

export async function GET(
  _request: NextRequest,
  { params }: { params: Promise<{ id: string }> },
) {
  const { id } = await params;
  const upstreamUrl = `${env.API_URL}/pipeline-execs/${id}`;
  const sessionToken = await getSessionToken();

  const upstreamResponse = await fetch(upstreamUrl, {
    headers: {
      Accept: "text/event-stream",
      Authorization: `Bearer ${sessionToken}`,
    },
  });

  if (!upstreamResponse.ok) {
    return new NextResponse("Failed to fetch SSE stream", {
      status: upstreamResponse.status,
    });
  }

  console.log(upstreamResponse.body);

  return new NextResponse(upstreamResponse.body, {
    headers: {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive",
    },
  });
}
