"use client";
import { zodResolver } from "@hookform/resolvers/zod";
import Link from "next/link";
import { useActionState, useEffect } from "react";
import { useForm } from "react-hook-form";
import type { z } from "zod";
import { loginAction } from "~/actions/login";
import { Button } from "~/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
} from "~/components/ui/card";
import {
  Form,
  FormControl,
  FormField,
  FormFieldMessage,
  FormItem,
  FormLabel,
  FormMessage,
} from "~/components/ui/form";
import { Input } from "~/components/ui/input";
import { loginSchema } from "~/lib/schemas";

export default function LoginPage() {
  const form = useForm<z.infer<typeof loginSchema>>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      email: "",
      password: "",
    },
  });

  const [formState, formAction] = useActionState(loginAction, {
    errors: {
      email: undefined,
      password: undefined,
    },
  });

  const setError = form.setError;

  useEffect(() => {
    for (const [field, message] of Object.entries(formState.errors)) {
      if (message === undefined || message.length === 0) {
        continue;
      }

      setError(field as Parameters<typeof setError>[0], {
        message: message[0],
      });
    }
  }, [setError, formState.errors]);

  return (
    <main className="flex items-center justify-center min-h-screen">
      <Card className="min-w-[400px]">
        <CardHeader>
          <h2 className="text-xl font-semibold">Login Page</h2>
        </CardHeader>

        <CardContent>
          <Form {...form}>
            <form
              className="flex flex-col items-center gap-4"
              action={formAction}
            >
              <FormField
                name="email"
                control={form.control}
                render={({ field }) => (
                  <FormItem className="min-w-full">
                    <FormLabel>Email</FormLabel>

                    <FormControl>
                      <Input {...field} autoComplete="email" />
                    </FormControl>

                    <FormFieldMessage />
                  </FormItem>
                )}
              />

              <FormField
                name="password"
                control={form.control}
                render={({ field }) => (
                  <FormItem className="min-w-full">
                    <FormLabel>Password</FormLabel>

                    <FormControl>
                      <Input
                        type="password"
                        {...field}
                        autoComplete="current-password"
                      />
                    </FormControl>

                    <FormFieldMessage />
                  </FormItem>
                )}
              />

              <FormMessage />

              <Button type="submit" className="mt-4 min-w-full justify-center">
                Sign in
              </Button>
            </form>
          </Form>
        </CardContent>

        <CardFooter>
          <Button variant="link" asChild>
            <Link href="/auth/registration" className="mt-4">
              Sign up instead
            </Link>
          </Button>
        </CardFooter>
      </Card>
    </main>
  );
}
