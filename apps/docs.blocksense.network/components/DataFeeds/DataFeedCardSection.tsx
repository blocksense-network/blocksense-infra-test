import React, { ReactNode } from 'react';
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
} from '@/components/common/Card';
import { DataFeedCardContent } from '@/components/DataFeeds/DataFeedCardContent';

type DataFeedCardSectionProps = {
  title: string;
  description: string;
  children: ReactNode;
};

export const DataFeedCardSection = ({
  title,
  description,
  children,
}: DataFeedCardSectionProps) => {
  return (
    <Card className="data-feed-card-section px-2 py-2">
      <CardHeader className="data-feed-card__header flex flex-col space-y-1.5 px-4 mt-3 mb-3">
        <CardTitle className="data-feed-card__title text-xl font-semibold text-gray-900">
          {title}
        </CardTitle>
        <CardDescription className="data-feed-card__description text-sm text-gray-500">
          {description}
        </CardDescription>
      </CardHeader>
      <DataFeedCardContent>{children}</DataFeedCardContent>
    </Card>
  );
};
