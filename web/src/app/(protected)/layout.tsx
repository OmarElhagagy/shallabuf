import { redirect } from "next/navigation";
import { Providers } from "~/app/(protected)/_providers";
import { getSessionToken } from "~/lib/auth";

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const session_token = await getSessionToken();

  if (!session_token) {
    return redirect("/auth/login");
  }

  return <Providers session_token={session_token}>{children}</Providers>;
}
