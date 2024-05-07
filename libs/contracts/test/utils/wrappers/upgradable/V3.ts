import { DataFeedStoreV3, UpgradeableProxy } from '../../../../typechain';
import { deployContract } from '../../helpers/common';
import { DataFeedStoreV3Wrapper } from '../dataFeedStore/V3';
import { UpgradeableProxyBaseWrapper } from './Base';

export class UpgradeableProxyDataFeedStoreV3Wrapper extends UpgradeableProxyBaseWrapper<
  DataFeedStoreV3,
  DataFeedStoreV3Wrapper
> {
  public override async init(...args: any[]) {
    this.implementation = new DataFeedStoreV3Wrapper();
    await this.implementation.init();

    this.contract = await deployContract<UpgradeableProxy>(
      'UpgradeableProxy',
      this.implementation.contract.target,
      ...args,
    );
  }

  public override getName(): string {
    return 'UpgradeableProxyDataFeedStoreV3';
  }
}
