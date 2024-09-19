import * as React from 'react';
import Link from 'next/link';

import { ColumnDef, Row } from '@tanstack/react-table';

import {
  EthereumAddress,
  explorerAddressUrls,
  NetworkName,
} from '@blocksense/base-utils/evm-utils';
import { ContractAddress } from '@/components/sol-contracts/ContractAddress';
import { CoreContract } from '@/src/deployed-contracts/types';
import { DataTableColumnHeader } from '@/components/ui/DataTable/DataTableColumnHeader';
import { Badge } from '@/components/ui/badge';

export const contractsColumnsTitles = {
  network: 'Network',
  contract: 'Contract',
  address: 'Address',
};

export const columns: ColumnDef<CoreContract>[] = [
  {
    accessorKey: 'contract',
    header: ({ column }) => (
      <DataTableColumnHeader
        column={column}
        title={contractsColumnsTitles['contract']}
        type={'string'}
        hasSort={false}
      />
    ),
  },
  {
    accessorKey: 'address',
    header: ({ column }) => (
      <DataTableColumnHeader
        column={column}
        title={contractsColumnsTitles['address']}
        type={'string'}
        hasSort={false}
      />
    ),
    cell: ({ row }) => (
      <ContractAddress address={row.getValue('address')} enableCopy />
    ),
  },
  {
    accessorKey: 'network',
    header: ({ column }) => (
      <DataTableColumnHeader
        column={column}
        title={contractsColumnsTitles['network']}
        type={'string'}
        hasSort={false}
      />
    ),
    cell: ({ row }) => <NetworkAddressExplorerLink row={row} />,
  },
];

type RowWrapper = {
  row: Row<CoreContract>;
};

const NetworkAddressExplorerLink = ({ row }: RowWrapper) => {
  const address = row.original.address;
  return (
    <aside className="space-x-2 space-y-1">
      {row.original.networks.map((network: NetworkName) => (
        <Badge
          key={network}
          variant="outline"
          className="border-solid border-slate-200 cursor-pointer m-0 text-primary-600 bold font-medium whitespace-nowrap"
        >
          <Link href={explorerAddressUrls[network](address as EthereumAddress)}>
            {network}
          </Link>
        </Badge>
      ))}
    </aside>
  );
};
