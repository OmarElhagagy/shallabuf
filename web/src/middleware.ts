import { NextResponse, type NextRequest } from "next/server";
import { env } from "./env";

const publicRoutes = ["/auth/login"];

export async function middleware(request: NextRequest) {
  const isPublic = publicRoutes.includes(request.nextUrl.pathname);

  if (!isPublic) {
    const sessionToken = request.cookies.get("session");

    if (!sessionToken) {
      return NextResponse.redirect(new URL("/auth/login", request.url));
    }

    const response = await fetch(`${env.API_URL}/auth/session`, {
      headers: {
        Authorization: `Bearer ${sessionToken.value}`,
      },
    });

    if (!response.ok) {
      return NextResponse.redirect(new URL("/auth/login", request.url));
    }
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    "/((?!_next/static|api|_next/image|favicon.ico|.*\\.(?:svg|png|jpg|jpeg|gif|webp)$).*)",
  ],
};
