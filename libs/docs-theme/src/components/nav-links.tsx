import cn from 'clsx';
import { ArrowRightIcon } from 'nextra/icons';
import type { Item } from 'nextra/normalize-pages';
import type { ReactElement } from 'react';
import { useConfig } from '../contexts';
import type { DocsThemeConfig } from '../index';
import { Anchor } from './anchor';
import { Tooltip } from './tooltip';

interface NavLinkProps {
  currentIndex: number;
  flatDirectories: Item[];
}

const classes = {
  link: cn(
    'nx-flex nx-max-w-[50%] nx-items-center nx-gap-1 nx-py-4 nx-text-sm nx-transition-colors [word-break:break-word] nx-font-semibold',
  ),
  icon: cn('nx-inline nx-h-5 nx-shrink-0'),
};

export const NavLinks = ({
  flatDirectories,
  currentIndex,
}: NavLinkProps): ReactElement | null => {
  const config = useConfig();
  const nav = config.navigation;
  const navigation: Exclude<DocsThemeConfig['navigation'], boolean> =
    typeof nav === 'boolean' ? { prev: nav, next: nav } : nav;
  let prev = navigation.prev && flatDirectories[currentIndex - 1];
  let next = navigation.next && flatDirectories[currentIndex + 1];

  if (prev && !prev.isUnderCurrentDocsTree) prev = false;
  if (next && !next.isUnderCurrentDocsTree) next = false;

  if (!prev && !next) return null;

  return (
    <div
      className={cn(
        'nx-mb-8 nx-flex nx-items-center nx-border-n nx-pt-8 dark:nx-border-neutral-800',
        'contrast-more:nx-border-neutral-400 dark:contrast-more:nx-border-neutral-400',
        'print:nx-hidden',
      )}
    >
      {prev && (
        <Anchor
          href={prev.route}
          className={cn(
            classes.link,
            'ltr:nx-mr-auto ltr:nx-pr-4 rtl:nx-pl-4 rtl:nx-mr-auto nx-text-black hover:nx-text-blue-900',
          )}
        >
          <ArrowRightIcon className={cn(classes.icon, 'ltr:nx-rotate-180')} />
          <span className="nx-block nx-text-sm nx-font-semibold nx-text-black hover:nx-text-blue-900">
            {prev.title}
          </span>
          <Tooltip>
            <Tooltip.Content>
              <span>Back to Previous</span>
            </Tooltip.Content>
            <span className="nx-hidden md:nx-block nx-text-xs nx-font-semibold nx-text-gray-600">
              Previous
            </span>
          </Tooltip>
        </Anchor>
      )}
      {next && (
        <Anchor
          href={next.route}
          className={cn(
            classes.link,
            'ltr:nx-ml-auto ltr:nx-pl-4 ltr:nx-text-right rtl:nx-mr-auto rtl:nx-pr-4 rtl:nx-text-left nx-text-blue-900 hover:nx-text-black',
          )}
        >
          <Tooltip>
            <Tooltip.Content>
              <span>Move Forward</span>
            </Tooltip.Content>
            <span className="nx-hidden md:nx-block nx-text-xs nx-font-semibold">
              Next
            </span>
          </Tooltip>
          <span className="nx-block nx-text-sm nx-font-semibold nx-text-black hover:nx-text-blue-900">
            {next.title}
          </span>
          <ArrowRightIcon className={cn(classes.icon, 'rtl:nx-rotate-180')} />
        </Anchor>
      )}
    </div>
  );
};
