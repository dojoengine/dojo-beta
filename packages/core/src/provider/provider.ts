import { EventEmitter } from "events";
import { StorageKey } from "../types";

interface IProvider {
    set_entity(component: number,
        key: StorageKey,
        offset: number,
        value: number[],
        calldata?: any[]): Promise<any>;
    get_entity(component: string, entity_id: string, offset: string, length: string): Promise<number>;
    get_entities(entites: any[]): Promise<number[]>;
}

export abstract class Provider extends EventEmitter implements IProvider {
    private readonly worldAddress: string;

    constructor(worldAddress: string) {
        super();
        this.worldAddress = worldAddress;
    }

    public abstract set_entity(component: number,
        key: StorageKey,
        offset: number,
        value: number[],
        calldata?: any[]): Promise<any>;
    public abstract get_entity(component: string, entity_id: string, offset: string, length: string): Promise<any>;
    public abstract get_entities(entites: any[]): Promise<any>;


    public getWorldAddress(): string {
        return this.worldAddress;
    }
}
