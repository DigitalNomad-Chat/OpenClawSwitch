<script setup lang="ts">
import { type HTMLAttributes, computed } from 'vue'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const badgeVariants = cva(
  'inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5 text-xs font-medium transition-colors',
  {
    variants: {
      variant: {
        default:
          'border-transparent bg-[var(--oc-accent)] text-white',
        secondary:
          'border-transparent bg-[var(--oc-card-elevated)] text-[var(--oc-text-secondary)]',
        destructive:
          'border-transparent bg-[var(--oc-danger)] text-white',
        outline:
          'border-[var(--oc-card-border)] text-[var(--oc-text-secondary)]',
        accent:
          'border-[var(--oc-accent)] bg-[color-mix(in_srgb,var(--oc-accent)_15%,transparent)] text-[var(--oc-accent)]',
        success:
          'border-transparent bg-[color-mix(in_srgb,var(--oc-success)_15%,transparent)] text-[var(--oc-success)]',
        warning:
          'border-transparent bg-[color-mix(in_srgb,var(--oc-warning)_15%,transparent)] text-[var(--oc-warning)]',
        purple:
          'border-transparent bg-[color-mix(in_srgb,var(--oc-purple)_15%,transparent)] text-[var(--oc-purple)]',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  }
)

interface Props {
  variant?: VariantProps<typeof badgeVariants>['variant']
  class?: HTMLAttributes['class']
}

const props = defineProps<Props>()

const classes = computed(() =>
  cn(badgeVariants({ variant: props.variant }), props.class)
)
</script>

<template>
  <span :class="classes">
    <slot />
  </span>
</template>
