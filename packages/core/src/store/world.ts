import { createStore } from 'zustand/vanilla'
import { ComponentsStore } from './components';
import { World, Manifest, CallData } from '../types';
import { SystemsStore } from './system';
import { RPCProvider } from '../provider';
import { Account, number } from 'starknet';
import { getEntityComponent, updateComponent, useEntityStore } from './entity';


export const WorldStore = createStore<World>(() => ({
    world: '',
    executor: ''
}))

/**
 * @param manifest dojo manifest
 * @returns
*/
export const registerWorld = (manifest: Manifest) => {
    WorldStore.setState(state => ({
        world: manifest.world,
        executor: manifest.executor
    }))

    ComponentsStore.setState(state => ({
        ...state,
        ...manifest.components
    }))

    SystemsStore.setState(state => ({
        ...state,
        ...manifest.systems
    }))
}

/**
 *  @returns world address  
*/
export const getWorld = () => {
    return WorldStore.getState()
}

// TODO: clean params
export async function execute(
    account: Account,
    provider: RPCProvider,
    system: string,
    component_data: any,
    call_data: number.BigNumberish[],
    entity_id: number,
    optimistic: boolean = false
) {

    // TODO: check system registered

    // get current entity by component
    const entity = getEntityComponent(entity_id, 'Position');

    // set component Store for Optimistic UI
    if (optimistic) updateComponent(entity_id, 'Position', component_data);


    try {
        const result = await provider.execute(account, system, call_data);

        return result;
    } catch (error) {
        // revert state if optimistic
        if (optimistic && entity) updateComponent(entity_id, system, entity.data);

        throw error;
    }
}