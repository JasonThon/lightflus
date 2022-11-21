import * as $protobuf from "protobufjs";
import Long = require("long");
/** Namespace apiserver. */
export namespace apiserver {

    /** Properties of a CreateResourceRequest. */
    interface ICreateResourceRequest {

        /** CreateResourceRequest namespace */
        namespace?: (string|null);

        /** CreateResourceRequest resourceType */
        resourceType?: (apiserver.ResourceTypeEnum|null);

        /** CreateResourceRequest dataflow */
        dataflow?: (apiserver.ICreateDataflowOptions|null);
    }

    /** Represents a CreateResourceRequest. */
    class CreateResourceRequest implements ICreateResourceRequest {

        /**
         * Constructs a new CreateResourceRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: apiserver.ICreateResourceRequest);

        /** CreateResourceRequest namespace. */
        public namespace: string;

        /** CreateResourceRequest resourceType. */
        public resourceType: apiserver.ResourceTypeEnum;

        /** CreateResourceRequest dataflow. */
        public dataflow?: (apiserver.ICreateDataflowOptions|null);

        /** CreateResourceRequest options. */
        public options?: "dataflow";

        /**
         * Creates a new CreateResourceRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns CreateResourceRequest instance
         */
        public static create(properties?: apiserver.ICreateResourceRequest): apiserver.CreateResourceRequest;

        /**
         * Encodes the specified CreateResourceRequest message. Does not implicitly {@link apiserver.CreateResourceRequest.verify|verify} messages.
         * @param message CreateResourceRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: apiserver.ICreateResourceRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CreateResourceRequest message, length delimited. Does not implicitly {@link apiserver.CreateResourceRequest.verify|verify} messages.
         * @param message CreateResourceRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: apiserver.ICreateResourceRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CreateResourceRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CreateResourceRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): apiserver.CreateResourceRequest;

        /**
         * Decodes a CreateResourceRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CreateResourceRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): apiserver.CreateResourceRequest;

        /**
         * Verifies a CreateResourceRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a CreateResourceRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CreateResourceRequest
         */
        public static fromObject(object: { [k: string]: any }): apiserver.CreateResourceRequest;

        /**
         * Creates a plain object from a CreateResourceRequest message. Also converts values to other types if specified.
         * @param message CreateResourceRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: apiserver.CreateResourceRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CreateResourceRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CreateResourceRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a CreateResourceResponse. */
    interface ICreateResourceResponse {

        /** CreateResourceResponse status */
        status?: (apiserver.ResourceStatusEnum|null);

        /** CreateResourceResponse resourceId */
        resourceId?: (common.IResourceId|null);

        /** CreateResourceResponse errorMsg */
        errorMsg?: (string|null);
    }

    /** Represents a CreateResourceResponse. */
    class CreateResourceResponse implements ICreateResourceResponse {

        /**
         * Constructs a new CreateResourceResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: apiserver.ICreateResourceResponse);

        /** CreateResourceResponse status. */
        public status: apiserver.ResourceStatusEnum;

        /** CreateResourceResponse resourceId. */
        public resourceId?: (common.IResourceId|null);

        /** CreateResourceResponse errorMsg. */
        public errorMsg: string;

        /**
         * Creates a new CreateResourceResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns CreateResourceResponse instance
         */
        public static create(properties?: apiserver.ICreateResourceResponse): apiserver.CreateResourceResponse;

        /**
         * Encodes the specified CreateResourceResponse message. Does not implicitly {@link apiserver.CreateResourceResponse.verify|verify} messages.
         * @param message CreateResourceResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: apiserver.ICreateResourceResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CreateResourceResponse message, length delimited. Does not implicitly {@link apiserver.CreateResourceResponse.verify|verify} messages.
         * @param message CreateResourceResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: apiserver.ICreateResourceResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CreateResourceResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CreateResourceResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): apiserver.CreateResourceResponse;

        /**
         * Decodes a CreateResourceResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CreateResourceResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): apiserver.CreateResourceResponse;

        /**
         * Verifies a CreateResourceResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a CreateResourceResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CreateResourceResponse
         */
        public static fromObject(object: { [k: string]: any }): apiserver.CreateResourceResponse;

        /**
         * Creates a plain object from a CreateResourceResponse message. Also converts values to other types if specified.
         * @param message CreateResourceResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: apiserver.CreateResourceResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CreateResourceResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CreateResourceResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** ResourceTypeEnum enum. */
    enum ResourceTypeEnum {
        RESOURCE_TYPE_ENUM_UNSPECIFIC = 0,
        RESOURCE_TYPE_ENUM_DATAFLOW = 1
    }

    /** ResourceStatusEnum enum. */
    enum ResourceStatusEnum {
        RESOURCE_STATUS_ENUM_UNSPECIFIC = 0,
        RESOURCE_STATUS_ENUM_STARTING = 1,
        RESOURCE_STATUS_ENUM_RUNNING = 2,
        RESOURCE_STATUS_ENUM_FAILURE = 3,
        RESOURCE_STATUS_ENUM_STOPPING = 4,
        RESOURCE_STATUS_ENUM_DELETED = 5
    }

    /** Properties of a CreateDataflowOptions. */
    interface ICreateDataflowOptions {

        /** CreateDataflowOptions dataflow */
        dataflow?: (common.IDataflow|null);
    }

    /** Represents a CreateDataflowOptions. */
    class CreateDataflowOptions implements ICreateDataflowOptions {

        /**
         * Constructs a new CreateDataflowOptions.
         * @param [properties] Properties to set
         */
        constructor(properties?: apiserver.ICreateDataflowOptions);

        /** CreateDataflowOptions dataflow. */
        public dataflow?: (common.IDataflow|null);

        /**
         * Creates a new CreateDataflowOptions instance using the specified properties.
         * @param [properties] Properties to set
         * @returns CreateDataflowOptions instance
         */
        public static create(properties?: apiserver.ICreateDataflowOptions): apiserver.CreateDataflowOptions;

        /**
         * Encodes the specified CreateDataflowOptions message. Does not implicitly {@link apiserver.CreateDataflowOptions.verify|verify} messages.
         * @param message CreateDataflowOptions message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: apiserver.ICreateDataflowOptions, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified CreateDataflowOptions message, length delimited. Does not implicitly {@link apiserver.CreateDataflowOptions.verify|verify} messages.
         * @param message CreateDataflowOptions message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: apiserver.ICreateDataflowOptions, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a CreateDataflowOptions message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns CreateDataflowOptions
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): apiserver.CreateDataflowOptions;

        /**
         * Decodes a CreateDataflowOptions message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns CreateDataflowOptions
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): apiserver.CreateDataflowOptions;

        /**
         * Verifies a CreateDataflowOptions message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a CreateDataflowOptions message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns CreateDataflowOptions
         */
        public static fromObject(object: { [k: string]: any }): apiserver.CreateDataflowOptions;

        /**
         * Creates a plain object from a CreateDataflowOptions message. Also converts values to other types if specified.
         * @param message CreateDataflowOptions
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: apiserver.CreateDataflowOptions, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this CreateDataflowOptions to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for CreateDataflowOptions
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ListResourcesRequest. */
    interface IListResourcesRequest {

        /** ListResourcesRequest namespace */
        namespace?: (string|null);

        /** ListResourcesRequest resourceType */
        resourceType?: (apiserver.ResourceTypeEnum|null);
    }

    /** Represents a ListResourcesRequest. */
    class ListResourcesRequest implements IListResourcesRequest {

        /**
         * Constructs a new ListResourcesRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: apiserver.IListResourcesRequest);

        /** ListResourcesRequest namespace. */
        public namespace: string;

        /** ListResourcesRequest resourceType. */
        public resourceType: apiserver.ResourceTypeEnum;

        /**
         * Creates a new ListResourcesRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ListResourcesRequest instance
         */
        public static create(properties?: apiserver.IListResourcesRequest): apiserver.ListResourcesRequest;

        /**
         * Encodes the specified ListResourcesRequest message. Does not implicitly {@link apiserver.ListResourcesRequest.verify|verify} messages.
         * @param message ListResourcesRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: apiserver.IListResourcesRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ListResourcesRequest message, length delimited. Does not implicitly {@link apiserver.ListResourcesRequest.verify|verify} messages.
         * @param message ListResourcesRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: apiserver.IListResourcesRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ListResourcesRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ListResourcesRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): apiserver.ListResourcesRequest;

        /**
         * Decodes a ListResourcesRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ListResourcesRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): apiserver.ListResourcesRequest;

        /**
         * Verifies a ListResourcesRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ListResourcesRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ListResourcesRequest
         */
        public static fromObject(object: { [k: string]: any }): apiserver.ListResourcesRequest;

        /**
         * Creates a plain object from a ListResourcesRequest message. Also converts values to other types if specified.
         * @param message ListResourcesRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: apiserver.ListResourcesRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ListResourcesRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ListResourcesRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ListResourcesResponse. */
    interface IListResourcesResponse {

        /** ListResourcesResponse resources */
        resources?: (apiserver.IResource[]|null);
    }

    /** Represents a ListResourcesResponse. */
    class ListResourcesResponse implements IListResourcesResponse {

        /**
         * Constructs a new ListResourcesResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: apiserver.IListResourcesResponse);

        /** ListResourcesResponse resources. */
        public resources: apiserver.IResource[];

        /**
         * Creates a new ListResourcesResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ListResourcesResponse instance
         */
        public static create(properties?: apiserver.IListResourcesResponse): apiserver.ListResourcesResponse;

        /**
         * Encodes the specified ListResourcesResponse message. Does not implicitly {@link apiserver.ListResourcesResponse.verify|verify} messages.
         * @param message ListResourcesResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: apiserver.IListResourcesResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ListResourcesResponse message, length delimited. Does not implicitly {@link apiserver.ListResourcesResponse.verify|verify} messages.
         * @param message ListResourcesResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: apiserver.IListResourcesResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ListResourcesResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ListResourcesResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): apiserver.ListResourcesResponse;

        /**
         * Decodes a ListResourcesResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ListResourcesResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): apiserver.ListResourcesResponse;

        /**
         * Verifies a ListResourcesResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ListResourcesResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ListResourcesResponse
         */
        public static fromObject(object: { [k: string]: any }): apiserver.ListResourcesResponse;

        /**
         * Creates a plain object from a ListResourcesResponse message. Also converts values to other types if specified.
         * @param message ListResourcesResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: apiserver.ListResourcesResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ListResourcesResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ListResourcesResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Resource. */
    interface IResource {

        /** Resource resourceId */
        resourceId?: (common.IResourceId|null);

        /** Resource resourceName */
        resourceName?: (string|null);

        /** Resource resourceType */
        resourceType?: (apiserver.ResourceTypeEnum|null);

        /** Resource status */
        status?: (apiserver.ResourceStatusEnum|null);
    }

    /** Represents a Resource. */
    class Resource implements IResource {

        /**
         * Constructs a new Resource.
         * @param [properties] Properties to set
         */
        constructor(properties?: apiserver.IResource);

        /** Resource resourceId. */
        public resourceId?: (common.IResourceId|null);

        /** Resource resourceName. */
        public resourceName: string;

        /** Resource resourceType. */
        public resourceType: apiserver.ResourceTypeEnum;

        /** Resource status. */
        public status: apiserver.ResourceStatusEnum;

        /**
         * Creates a new Resource instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Resource instance
         */
        public static create(properties?: apiserver.IResource): apiserver.Resource;

        /**
         * Encodes the specified Resource message. Does not implicitly {@link apiserver.Resource.verify|verify} messages.
         * @param message Resource message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: apiserver.IResource, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Resource message, length delimited. Does not implicitly {@link apiserver.Resource.verify|verify} messages.
         * @param message Resource message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: apiserver.IResource, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Resource message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Resource
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): apiserver.Resource;

        /**
         * Decodes a Resource message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Resource
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): apiserver.Resource;

        /**
         * Verifies a Resource message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Resource message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Resource
         */
        public static fromObject(object: { [k: string]: any }): apiserver.Resource;

        /**
         * Creates a plain object from a Resource message. Also converts values to other types if specified.
         * @param message Resource
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: apiserver.Resource, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Resource to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Resource
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a GetResourceRequest. */
    interface IGetResourceRequest {

        /** GetResourceRequest resourceId */
        resourceId?: (common.IResourceId|null);
    }

    /** Represents a GetResourceRequest. */
    class GetResourceRequest implements IGetResourceRequest {

        /**
         * Constructs a new GetResourceRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: apiserver.IGetResourceRequest);

        /** GetResourceRequest resourceId. */
        public resourceId?: (common.IResourceId|null);

        /**
         * Creates a new GetResourceRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns GetResourceRequest instance
         */
        public static create(properties?: apiserver.IGetResourceRequest): apiserver.GetResourceRequest;

        /**
         * Encodes the specified GetResourceRequest message. Does not implicitly {@link apiserver.GetResourceRequest.verify|verify} messages.
         * @param message GetResourceRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: apiserver.IGetResourceRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified GetResourceRequest message, length delimited. Does not implicitly {@link apiserver.GetResourceRequest.verify|verify} messages.
         * @param message GetResourceRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: apiserver.IGetResourceRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a GetResourceRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns GetResourceRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): apiserver.GetResourceRequest;

        /**
         * Decodes a GetResourceRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns GetResourceRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): apiserver.GetResourceRequest;

        /**
         * Verifies a GetResourceRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a GetResourceRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns GetResourceRequest
         */
        public static fromObject(object: { [k: string]: any }): apiserver.GetResourceRequest;

        /**
         * Creates a plain object from a GetResourceRequest message. Also converts values to other types if specified.
         * @param message GetResourceRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: apiserver.GetResourceRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this GetResourceRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for GetResourceRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a GetResourceResponse. */
    interface IGetResourceResponse {

        /** GetResourceResponse resource */
        resource?: (apiserver.IResource|null);
    }

    /** Represents a GetResourceResponse. */
    class GetResourceResponse implements IGetResourceResponse {

        /**
         * Constructs a new GetResourceResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: apiserver.IGetResourceResponse);

        /** GetResourceResponse resource. */
        public resource?: (apiserver.IResource|null);

        /**
         * Creates a new GetResourceResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns GetResourceResponse instance
         */
        public static create(properties?: apiserver.IGetResourceResponse): apiserver.GetResourceResponse;

        /**
         * Encodes the specified GetResourceResponse message. Does not implicitly {@link apiserver.GetResourceResponse.verify|verify} messages.
         * @param message GetResourceResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: apiserver.IGetResourceResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified GetResourceResponse message, length delimited. Does not implicitly {@link apiserver.GetResourceResponse.verify|verify} messages.
         * @param message GetResourceResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: apiserver.IGetResourceResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a GetResourceResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns GetResourceResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): apiserver.GetResourceResponse;

        /**
         * Decodes a GetResourceResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns GetResourceResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): apiserver.GetResourceResponse;

        /**
         * Verifies a GetResourceResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a GetResourceResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns GetResourceResponse
         */
        public static fromObject(object: { [k: string]: any }): apiserver.GetResourceResponse;

        /**
         * Creates a plain object from a GetResourceResponse message. Also converts values to other types if specified.
         * @param message GetResourceResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: apiserver.GetResourceResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this GetResourceResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for GetResourceResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a DeleteResourceRequest. */
    interface IDeleteResourceRequest {

        /** DeleteResourceRequest resourceId */
        resourceId?: (common.IResourceId|null);
    }

    /** Represents a DeleteResourceRequest. */
    class DeleteResourceRequest implements IDeleteResourceRequest {

        /**
         * Constructs a new DeleteResourceRequest.
         * @param [properties] Properties to set
         */
        constructor(properties?: apiserver.IDeleteResourceRequest);

        /** DeleteResourceRequest resourceId. */
        public resourceId?: (common.IResourceId|null);

        /**
         * Creates a new DeleteResourceRequest instance using the specified properties.
         * @param [properties] Properties to set
         * @returns DeleteResourceRequest instance
         */
        public static create(properties?: apiserver.IDeleteResourceRequest): apiserver.DeleteResourceRequest;

        /**
         * Encodes the specified DeleteResourceRequest message. Does not implicitly {@link apiserver.DeleteResourceRequest.verify|verify} messages.
         * @param message DeleteResourceRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: apiserver.IDeleteResourceRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified DeleteResourceRequest message, length delimited. Does not implicitly {@link apiserver.DeleteResourceRequest.verify|verify} messages.
         * @param message DeleteResourceRequest message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: apiserver.IDeleteResourceRequest, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a DeleteResourceRequest message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns DeleteResourceRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): apiserver.DeleteResourceRequest;

        /**
         * Decodes a DeleteResourceRequest message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns DeleteResourceRequest
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): apiserver.DeleteResourceRequest;

        /**
         * Verifies a DeleteResourceRequest message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a DeleteResourceRequest message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns DeleteResourceRequest
         */
        public static fromObject(object: { [k: string]: any }): apiserver.DeleteResourceRequest;

        /**
         * Creates a plain object from a DeleteResourceRequest message. Also converts values to other types if specified.
         * @param message DeleteResourceRequest
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: apiserver.DeleteResourceRequest, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this DeleteResourceRequest to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for DeleteResourceRequest
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a DeleteResourceResponse. */
    interface IDeleteResourceResponse {

        /** DeleteResourceResponse resource */
        resource?: (apiserver.IResource|null);
    }

    /** Represents a DeleteResourceResponse. */
    class DeleteResourceResponse implements IDeleteResourceResponse {

        /**
         * Constructs a new DeleteResourceResponse.
         * @param [properties] Properties to set
         */
        constructor(properties?: apiserver.IDeleteResourceResponse);

        /** DeleteResourceResponse resource. */
        public resource?: (apiserver.IResource|null);

        /**
         * Creates a new DeleteResourceResponse instance using the specified properties.
         * @param [properties] Properties to set
         * @returns DeleteResourceResponse instance
         */
        public static create(properties?: apiserver.IDeleteResourceResponse): apiserver.DeleteResourceResponse;

        /**
         * Encodes the specified DeleteResourceResponse message. Does not implicitly {@link apiserver.DeleteResourceResponse.verify|verify} messages.
         * @param message DeleteResourceResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: apiserver.IDeleteResourceResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified DeleteResourceResponse message, length delimited. Does not implicitly {@link apiserver.DeleteResourceResponse.verify|verify} messages.
         * @param message DeleteResourceResponse message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: apiserver.IDeleteResourceResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a DeleteResourceResponse message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns DeleteResourceResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): apiserver.DeleteResourceResponse;

        /**
         * Decodes a DeleteResourceResponse message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns DeleteResourceResponse
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): apiserver.DeleteResourceResponse;

        /**
         * Verifies a DeleteResourceResponse message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a DeleteResourceResponse message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns DeleteResourceResponse
         */
        public static fromObject(object: { [k: string]: any }): apiserver.DeleteResourceResponse;

        /**
         * Creates a plain object from a DeleteResourceResponse message. Also converts values to other types if specified.
         * @param message DeleteResourceResponse
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: apiserver.DeleteResourceResponse, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this DeleteResourceResponse to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for DeleteResourceResponse
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }
}

/** Namespace common. */
export namespace common {

    /** Properties of a DataflowMeta. */
    interface IDataflowMeta {

        /** DataflowMeta center */
        center?: (number|null);

        /** DataflowMeta neighbors */
        neighbors?: (number[]|null);
    }

    /** StreamGraph metadata, it stores the structural information of a stream graph */
    class DataflowMeta implements IDataflowMeta {

        /**
         * Constructs a new DataflowMeta.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IDataflowMeta);

        /** DataflowMeta center. */
        public center: number;

        /** DataflowMeta neighbors. */
        public neighbors: number[];

        /**
         * Creates a new DataflowMeta instance using the specified properties.
         * @param [properties] Properties to set
         * @returns DataflowMeta instance
         */
        public static create(properties?: common.IDataflowMeta): common.DataflowMeta;

        /**
         * Encodes the specified DataflowMeta message. Does not implicitly {@link common.DataflowMeta.verify|verify} messages.
         * @param message DataflowMeta message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IDataflowMeta, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified DataflowMeta message, length delimited. Does not implicitly {@link common.DataflowMeta.verify|verify} messages.
         * @param message DataflowMeta message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IDataflowMeta, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a DataflowMeta message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns DataflowMeta
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.DataflowMeta;

        /**
         * Decodes a DataflowMeta message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns DataflowMeta
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.DataflowMeta;

        /**
         * Verifies a DataflowMeta message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a DataflowMeta message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns DataflowMeta
         */
        public static fromObject(object: { [k: string]: any }): common.DataflowMeta;

        /**
         * Creates a plain object from a DataflowMeta message. Also converts values to other types if specified.
         * @param message DataflowMeta
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.DataflowMeta, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this DataflowMeta to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for DataflowMeta
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of an OperatorInfo. */
    interface IOperatorInfo {

        /** OperatorInfo operatorId */
        operatorId?: (number|null);

        /** OperatorInfo hostAddr */
        hostAddr?: (common.IHostAddr|null);

        /** OperatorInfo upstreams */
        upstreams?: (number[]|null);

        /** OperatorInfo source */
        source?: (common.ISource|null);

        /** OperatorInfo sink */
        sink?: (common.ISink|null);

        /** OperatorInfo mapper */
        mapper?: (common.IMapper|null);

        /** OperatorInfo filter */
        filter?: (common.IFilter|null);

        /** OperatorInfo keyBy */
        keyBy?: (common.IKeyBy|null);

        /** OperatorInfo reducer */
        reducer?: (common.IReducer|null);

        /** OperatorInfo flatMap */
        flatMap?: (common.IFlatMap|null);

        /** OperatorInfo window */
        window?: (common.IWindow|null);

        /** OperatorInfo trigger */
        trigger?: (common.ITrigger|null);
    }

    /** OperatorInfo, stores detail information of an operator */
    class OperatorInfo implements IOperatorInfo {

        /**
         * Constructs a new OperatorInfo.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IOperatorInfo);

        /** OperatorInfo operatorId. */
        public operatorId: number;

        /** OperatorInfo hostAddr. */
        public hostAddr?: (common.IHostAddr|null);

        /** OperatorInfo upstreams. */
        public upstreams: number[];

        /** OperatorInfo source. */
        public source?: (common.ISource|null);

        /** OperatorInfo sink. */
        public sink?: (common.ISink|null);

        /** OperatorInfo mapper. */
        public mapper?: (common.IMapper|null);

        /** OperatorInfo filter. */
        public filter?: (common.IFilter|null);

        /** OperatorInfo keyBy. */
        public keyBy?: (common.IKeyBy|null);

        /** OperatorInfo reducer. */
        public reducer?: (common.IReducer|null);

        /** OperatorInfo flatMap. */
        public flatMap?: (common.IFlatMap|null);

        /** OperatorInfo window. */
        public window?: (common.IWindow|null);

        /** OperatorInfo trigger. */
        public trigger?: (common.ITrigger|null);

        /** OperatorInfo details. */
        public details?: ("source"|"sink"|"mapper"|"filter"|"keyBy"|"reducer"|"flatMap"|"window"|"trigger");

        /**
         * Creates a new OperatorInfo instance using the specified properties.
         * @param [properties] Properties to set
         * @returns OperatorInfo instance
         */
        public static create(properties?: common.IOperatorInfo): common.OperatorInfo;

        /**
         * Encodes the specified OperatorInfo message. Does not implicitly {@link common.OperatorInfo.verify|verify} messages.
         * @param message OperatorInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IOperatorInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified OperatorInfo message, length delimited. Does not implicitly {@link common.OperatorInfo.verify|verify} messages.
         * @param message OperatorInfo message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IOperatorInfo, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes an OperatorInfo message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns OperatorInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.OperatorInfo;

        /**
         * Decodes an OperatorInfo message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns OperatorInfo
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.OperatorInfo;

        /**
         * Verifies an OperatorInfo message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates an OperatorInfo message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns OperatorInfo
         */
        public static fromObject(object: { [k: string]: any }): common.OperatorInfo;

        /**
         * Creates a plain object from an OperatorInfo message. Also converts values to other types if specified.
         * @param message OperatorInfo
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.OperatorInfo, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this OperatorInfo to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for OperatorInfo
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Reducer. */
    interface IReducer {

        /** Reducer func */
        func?: (common.IFunc|null);
    }

    /** Represents a Reducer. */
    class Reducer implements IReducer {

        /**
         * Constructs a new Reducer.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IReducer);

        /** Reducer func. */
        public func?: (common.IFunc|null);

        /** Reducer value. */
        public value?: "func";

        /**
         * Creates a new Reducer instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Reducer instance
         */
        public static create(properties?: common.IReducer): common.Reducer;

        /**
         * Encodes the specified Reducer message. Does not implicitly {@link common.Reducer.verify|verify} messages.
         * @param message Reducer message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IReducer, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Reducer message, length delimited. Does not implicitly {@link common.Reducer.verify|verify} messages.
         * @param message Reducer message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IReducer, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Reducer message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Reducer
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Reducer;

        /**
         * Decodes a Reducer message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Reducer
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Reducer;

        /**
         * Verifies a Reducer message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Reducer message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Reducer
         */
        public static fromObject(object: { [k: string]: any }): common.Reducer;

        /**
         * Creates a plain object from a Reducer message. Also converts values to other types if specified.
         * @param message Reducer
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Reducer, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Reducer to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Reducer
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a FlatMap. */
    interface IFlatMap {

        /** FlatMap func */
        func?: (common.IFunc|null);
    }

    /** Represents a FlatMap. */
    class FlatMap implements IFlatMap {

        /**
         * Constructs a new FlatMap.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IFlatMap);

        /** FlatMap func. */
        public func?: (common.IFunc|null);

        /** FlatMap value. */
        public value?: "func";

        /**
         * Creates a new FlatMap instance using the specified properties.
         * @param [properties] Properties to set
         * @returns FlatMap instance
         */
        public static create(properties?: common.IFlatMap): common.FlatMap;

        /**
         * Encodes the specified FlatMap message. Does not implicitly {@link common.FlatMap.verify|verify} messages.
         * @param message FlatMap message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IFlatMap, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified FlatMap message, length delimited. Does not implicitly {@link common.FlatMap.verify|verify} messages.
         * @param message FlatMap message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IFlatMap, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a FlatMap message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns FlatMap
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.FlatMap;

        /**
         * Decodes a FlatMap message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns FlatMap
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.FlatMap;

        /**
         * Verifies a FlatMap message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a FlatMap message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns FlatMap
         */
        public static fromObject(object: { [k: string]: any }): common.FlatMap;

        /**
         * Creates a plain object from a FlatMap message. Also converts values to other types if specified.
         * @param message FlatMap
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.FlatMap, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this FlatMap to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for FlatMap
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Join. */
    interface IJoin {

        /** Join streamJoin */
        streamJoin?: (common.Join.IStreamJoin|null);
    }

    /** Represents a Join. */
    class Join implements IJoin {

        /**
         * Constructs a new Join.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IJoin);

        /** Join streamJoin. */
        public streamJoin?: (common.Join.IStreamJoin|null);

        /** Join value. */
        public value?: "streamJoin";

        /**
         * Creates a new Join instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Join instance
         */
        public static create(properties?: common.IJoin): common.Join;

        /**
         * Encodes the specified Join message. Does not implicitly {@link common.Join.verify|verify} messages.
         * @param message Join message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IJoin, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Join message, length delimited. Does not implicitly {@link common.Join.verify|verify} messages.
         * @param message Join message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IJoin, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Join message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Join
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Join;

        /**
         * Decodes a Join message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Join
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Join;

        /**
         * Verifies a Join message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Join message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Join
         */
        public static fromObject(object: { [k: string]: any }): common.Join;

        /**
         * Creates a plain object from a Join message. Also converts values to other types if specified.
         * @param message Join
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Join, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Join to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Join
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    namespace Join {

        /** Properties of a StreamJoin. */
        interface IStreamJoin {

            /** StreamJoin operatorId */
            operatorId?: (number|null);

            /** StreamJoin func */
            func?: (common.IFunc|null);
        }

        /** Represents a StreamJoin. */
        class StreamJoin implements IStreamJoin {

            /**
             * Constructs a new StreamJoin.
             * @param [properties] Properties to set
             */
            constructor(properties?: common.Join.IStreamJoin);

            /** StreamJoin operatorId. */
            public operatorId: number;

            /** StreamJoin func. */
            public func?: (common.IFunc|null);

            /**
             * Creates a new StreamJoin instance using the specified properties.
             * @param [properties] Properties to set
             * @returns StreamJoin instance
             */
            public static create(properties?: common.Join.IStreamJoin): common.Join.StreamJoin;

            /**
             * Encodes the specified StreamJoin message. Does not implicitly {@link common.Join.StreamJoin.verify|verify} messages.
             * @param message StreamJoin message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: common.Join.IStreamJoin, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified StreamJoin message, length delimited. Does not implicitly {@link common.Join.StreamJoin.verify|verify} messages.
             * @param message StreamJoin message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: common.Join.IStreamJoin, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a StreamJoin message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns StreamJoin
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Join.StreamJoin;

            /**
             * Decodes a StreamJoin message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns StreamJoin
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Join.StreamJoin;

            /**
             * Verifies a StreamJoin message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a StreamJoin message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns StreamJoin
             */
            public static fromObject(object: { [k: string]: any }): common.Join.StreamJoin;

            /**
             * Creates a plain object from a StreamJoin message. Also converts values to other types if specified.
             * @param message StreamJoin
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: common.Join.StreamJoin, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this StreamJoin to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for StreamJoin
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }
    }

    /** Properties of a Mapper. */
    interface IMapper {

        /** Mapper func */
        func?: (common.IFunc|null);
    }

    /** Represents a Mapper. */
    class Mapper implements IMapper {

        /**
         * Constructs a new Mapper.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IMapper);

        /** Mapper func. */
        public func?: (common.IFunc|null);

        /** Mapper value. */
        public value?: "func";

        /**
         * Creates a new Mapper instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Mapper instance
         */
        public static create(properties?: common.IMapper): common.Mapper;

        /**
         * Encodes the specified Mapper message. Does not implicitly {@link common.Mapper.verify|verify} messages.
         * @param message Mapper message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IMapper, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Mapper message, length delimited. Does not implicitly {@link common.Mapper.verify|verify} messages.
         * @param message Mapper message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IMapper, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Mapper message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Mapper
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Mapper;

        /**
         * Decodes a Mapper message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Mapper
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Mapper;

        /**
         * Verifies a Mapper message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Mapper message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Mapper
         */
        public static fromObject(object: { [k: string]: any }): common.Mapper;

        /**
         * Creates a plain object from a Mapper message. Also converts values to other types if specified.
         * @param message Mapper
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Mapper, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Mapper to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Mapper
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Func. */
    interface IFunc {

        /** Func function */
        "function"?: (string|null);
    }

    /** Represents a Func. */
    class Func implements IFunc {

        /**
         * Constructs a new Func.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IFunc);

        /** Func function. */
        public function: string;

        /**
         * Creates a new Func instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Func instance
         */
        public static create(properties?: common.IFunc): common.Func;

        /**
         * Encodes the specified Func message. Does not implicitly {@link common.Func.verify|verify} messages.
         * @param message Func message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IFunc, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Func message, length delimited. Does not implicitly {@link common.Func.verify|verify} messages.
         * @param message Func message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IFunc, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Func message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Func
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Func;

        /**
         * Decodes a Func message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Func
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Func;

        /**
         * Verifies a Func message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Func message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Func
         */
        public static fromObject(object: { [k: string]: any }): common.Func;

        /**
         * Creates a plain object from a Func message. Also converts values to other types if specified.
         * @param message Func
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Func, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Func to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Func
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Filter. */
    interface IFilter {

        /** Filter func */
        func?: (common.IFunc|null);
    }

    /** Represents a Filter. */
    class Filter implements IFilter {

        /**
         * Constructs a new Filter.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IFilter);

        /** Filter func. */
        public func?: (common.IFunc|null);

        /** Filter value. */
        public value?: "func";

        /**
         * Creates a new Filter instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Filter instance
         */
        public static create(properties?: common.IFilter): common.Filter;

        /**
         * Encodes the specified Filter message. Does not implicitly {@link common.Filter.verify|verify} messages.
         * @param message Filter message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IFilter, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Filter message, length delimited. Does not implicitly {@link common.Filter.verify|verify} messages.
         * @param message Filter message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IFilter, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Filter message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Filter
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Filter;

        /**
         * Decodes a Filter message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Filter
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Filter;

        /**
         * Verifies a Filter message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Filter message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Filter
         */
        public static fromObject(object: { [k: string]: any }): common.Filter;

        /**
         * Creates a plain object from a Filter message. Also converts values to other types if specified.
         * @param message Filter
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Filter, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Filter to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Filter
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a KeyBy. */
    interface IKeyBy {

        /** KeyBy func */
        func?: (common.IFunc|null);
    }

    /** Represents a KeyBy. */
    class KeyBy implements IKeyBy {

        /**
         * Constructs a new KeyBy.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IKeyBy);

        /** KeyBy func. */
        public func?: (common.IFunc|null);

        /** KeyBy value. */
        public value?: "func";

        /**
         * Creates a new KeyBy instance using the specified properties.
         * @param [properties] Properties to set
         * @returns KeyBy instance
         */
        public static create(properties?: common.IKeyBy): common.KeyBy;

        /**
         * Encodes the specified KeyBy message. Does not implicitly {@link common.KeyBy.verify|verify} messages.
         * @param message KeyBy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IKeyBy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified KeyBy message, length delimited. Does not implicitly {@link common.KeyBy.verify|verify} messages.
         * @param message KeyBy message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IKeyBy, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a KeyBy message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns KeyBy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.KeyBy;

        /**
         * Decodes a KeyBy message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns KeyBy
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.KeyBy;

        /**
         * Verifies a KeyBy message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a KeyBy message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns KeyBy
         */
        public static fromObject(object: { [k: string]: any }): common.KeyBy;

        /**
         * Creates a plain object from a KeyBy message. Also converts values to other types if specified.
         * @param message KeyBy
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.KeyBy, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this KeyBy to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for KeyBy
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Sink. */
    interface ISink {

        /** Sink kafka */
        kafka?: (common.IKafkaDesc|null);

        /** Sink mysql */
        mysql?: (common.IMysqlDesc|null);

        /** Sink redis */
        redis?: (common.IRedisDesc|null);
    }

    /** Represents a Sink. */
    class Sink implements ISink {

        /**
         * Constructs a new Sink.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.ISink);

        /** Sink kafka. */
        public kafka?: (common.IKafkaDesc|null);

        /** Sink mysql. */
        public mysql?: (common.IMysqlDesc|null);

        /** Sink redis. */
        public redis?: (common.IRedisDesc|null);

        /** Sink desc. */
        public desc?: ("kafka"|"mysql"|"redis");

        /**
         * Creates a new Sink instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Sink instance
         */
        public static create(properties?: common.ISink): common.Sink;

        /**
         * Encodes the specified Sink message. Does not implicitly {@link common.Sink.verify|verify} messages.
         * @param message Sink message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.ISink, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Sink message, length delimited. Does not implicitly {@link common.Sink.verify|verify} messages.
         * @param message Sink message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.ISink, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Sink message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Sink
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Sink;

        /**
         * Decodes a Sink message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Sink
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Sink;

        /**
         * Verifies a Sink message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Sink message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Sink
         */
        public static fromObject(object: { [k: string]: any }): common.Sink;

        /**
         * Creates a plain object from a Sink message. Also converts values to other types if specified.
         * @param message Sink
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Sink, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Sink to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Sink
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a ConstOp. */
    interface IConstOp {

        /** ConstOp value */
        value?: (Uint8Array|null);

        /** ConstOp operatorId */
        operatorId?: (number|null);
    }

    /** Constant operator */
    class ConstOp implements IConstOp {

        /**
         * Constructs a new ConstOp.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IConstOp);

        /** ConstOp value. */
        public value: Uint8Array;

        /** ConstOp operatorId. */
        public operatorId: number;

        /**
         * Creates a new ConstOp instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ConstOp instance
         */
        public static create(properties?: common.IConstOp): common.ConstOp;

        /**
         * Encodes the specified ConstOp message. Does not implicitly {@link common.ConstOp.verify|verify} messages.
         * @param message ConstOp message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IConstOp, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ConstOp message, length delimited. Does not implicitly {@link common.ConstOp.verify|verify} messages.
         * @param message ConstOp message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IConstOp, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ConstOp message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ConstOp
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.ConstOp;

        /**
         * Decodes a ConstOp message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ConstOp
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.ConstOp;

        /**
         * Verifies a ConstOp message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ConstOp message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ConstOp
         */
        public static fromObject(object: { [k: string]: any }): common.ConstOp;

        /**
         * Creates a plain object from a ConstOp message. Also converts values to other types if specified.
         * @param message ConstOp
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.ConstOp, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ConstOp to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ConstOp
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Source. */
    interface ISource {

        /** Source kafka */
        kafka?: (common.IKafkaDesc|null);
    }

    /** Represents a Source. */
    class Source implements ISource {

        /**
         * Constructs a new Source.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.ISource);

        /** Source kafka. */
        public kafka?: (common.IKafkaDesc|null);

        /** Source desc. */
        public desc?: "kafka";

        /**
         * Creates a new Source instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Source instance
         */
        public static create(properties?: common.ISource): common.Source;

        /**
         * Encodes the specified Source message. Does not implicitly {@link common.Source.verify|verify} messages.
         * @param message Source message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.ISource, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Source message, length delimited. Does not implicitly {@link common.Source.verify|verify} messages.
         * @param message Source message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.ISource, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Source message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Source
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Source;

        /**
         * Decodes a Source message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Source
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Source;

        /**
         * Verifies a Source message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Source message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Source
         */
        public static fromObject(object: { [k: string]: any }): common.Source;

        /**
         * Creates a plain object from a Source message. Also converts values to other types if specified.
         * @param message Source
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Source, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Source to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Source
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a KafkaDesc. */
    interface IKafkaDesc {

        /** KafkaDesc brokers */
        brokers?: (string[]|null);

        /** KafkaDesc topic */
        topic?: (string|null);

        /** KafkaDesc opts */
        opts?: (common.KafkaDesc.IKafkaOptions|null);

        /** KafkaDesc dataType */
        dataType?: (common.DataTypeEnum|null);
    }

    /** Represents a KafkaDesc. */
    class KafkaDesc implements IKafkaDesc {

        /**
         * Constructs a new KafkaDesc.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IKafkaDesc);

        /** KafkaDesc brokers. */
        public brokers: string[];

        /** KafkaDesc topic. */
        public topic: string;

        /** KafkaDesc opts. */
        public opts?: (common.KafkaDesc.IKafkaOptions|null);

        /** KafkaDesc dataType. */
        public dataType: common.DataTypeEnum;

        /**
         * Creates a new KafkaDesc instance using the specified properties.
         * @param [properties] Properties to set
         * @returns KafkaDesc instance
         */
        public static create(properties?: common.IKafkaDesc): common.KafkaDesc;

        /**
         * Encodes the specified KafkaDesc message. Does not implicitly {@link common.KafkaDesc.verify|verify} messages.
         * @param message KafkaDesc message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IKafkaDesc, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified KafkaDesc message, length delimited. Does not implicitly {@link common.KafkaDesc.verify|verify} messages.
         * @param message KafkaDesc message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IKafkaDesc, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a KafkaDesc message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns KafkaDesc
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.KafkaDesc;

        /**
         * Decodes a KafkaDesc message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns KafkaDesc
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.KafkaDesc;

        /**
         * Verifies a KafkaDesc message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a KafkaDesc message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns KafkaDesc
         */
        public static fromObject(object: { [k: string]: any }): common.KafkaDesc;

        /**
         * Creates a plain object from a KafkaDesc message. Also converts values to other types if specified.
         * @param message KafkaDesc
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.KafkaDesc, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this KafkaDesc to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for KafkaDesc
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    namespace KafkaDesc {

        /** Properties of a KafkaOptions. */
        interface IKafkaOptions {

            /** KafkaOptions group */
            group?: (string|null);

            /** KafkaOptions partition */
            partition?: (number|null);
        }

        /** Represents a KafkaOptions. */
        class KafkaOptions implements IKafkaOptions {

            /**
             * Constructs a new KafkaOptions.
             * @param [properties] Properties to set
             */
            constructor(properties?: common.KafkaDesc.IKafkaOptions);

            /** KafkaOptions group. */
            public group?: (string|null);

            /** KafkaOptions partition. */
            public partition?: (number|null);

            /** KafkaOptions opt. */
            public opt?: ("group"|"partition");

            /**
             * Creates a new KafkaOptions instance using the specified properties.
             * @param [properties] Properties to set
             * @returns KafkaOptions instance
             */
            public static create(properties?: common.KafkaDesc.IKafkaOptions): common.KafkaDesc.KafkaOptions;

            /**
             * Encodes the specified KafkaOptions message. Does not implicitly {@link common.KafkaDesc.KafkaOptions.verify|verify} messages.
             * @param message KafkaOptions message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: common.KafkaDesc.IKafkaOptions, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified KafkaOptions message, length delimited. Does not implicitly {@link common.KafkaDesc.KafkaOptions.verify|verify} messages.
             * @param message KafkaOptions message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: common.KafkaDesc.IKafkaOptions, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a KafkaOptions message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns KafkaOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.KafkaDesc.KafkaOptions;

            /**
             * Decodes a KafkaOptions message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns KafkaOptions
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.KafkaDesc.KafkaOptions;

            /**
             * Verifies a KafkaOptions message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a KafkaOptions message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns KafkaOptions
             */
            public static fromObject(object: { [k: string]: any }): common.KafkaDesc.KafkaOptions;

            /**
             * Creates a plain object from a KafkaOptions message. Also converts values to other types if specified.
             * @param message KafkaOptions
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: common.KafkaDesc.KafkaOptions, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this KafkaOptions to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for KafkaOptions
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }
    }

    /** Properties of a MysqlDesc. */
    interface IMysqlDesc {

        /** MysqlDesc connectionOpts */
        connectionOpts?: (common.MysqlDesc.IConnectionOpts|null);

        /** MysqlDesc statement */
        statement?: (common.MysqlDesc.IStatement|null);
    }

    /** Represents a MysqlDesc. */
    class MysqlDesc implements IMysqlDesc {

        /**
         * Constructs a new MysqlDesc.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IMysqlDesc);

        /** MysqlDesc connectionOpts. */
        public connectionOpts?: (common.MysqlDesc.IConnectionOpts|null);

        /** MysqlDesc statement. */
        public statement?: (common.MysqlDesc.IStatement|null);

        /**
         * Creates a new MysqlDesc instance using the specified properties.
         * @param [properties] Properties to set
         * @returns MysqlDesc instance
         */
        public static create(properties?: common.IMysqlDesc): common.MysqlDesc;

        /**
         * Encodes the specified MysqlDesc message. Does not implicitly {@link common.MysqlDesc.verify|verify} messages.
         * @param message MysqlDesc message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IMysqlDesc, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified MysqlDesc message, length delimited. Does not implicitly {@link common.MysqlDesc.verify|verify} messages.
         * @param message MysqlDesc message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IMysqlDesc, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a MysqlDesc message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns MysqlDesc
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.MysqlDesc;

        /**
         * Decodes a MysqlDesc message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns MysqlDesc
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.MysqlDesc;

        /**
         * Verifies a MysqlDesc message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a MysqlDesc message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns MysqlDesc
         */
        public static fromObject(object: { [k: string]: any }): common.MysqlDesc;

        /**
         * Creates a plain object from a MysqlDesc message. Also converts values to other types if specified.
         * @param message MysqlDesc
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.MysqlDesc, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this MysqlDesc to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for MysqlDesc
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    namespace MysqlDesc {

        /** Properties of a ConnectionOpts. */
        interface IConnectionOpts {

            /** ConnectionOpts host */
            host?: (string|null);

            /** ConnectionOpts username */
            username?: (string|null);

            /** ConnectionOpts password */
            password?: (string|null);

            /** ConnectionOpts database */
            database?: (string|null);
        }

        /** Represents a ConnectionOpts. */
        class ConnectionOpts implements IConnectionOpts {

            /**
             * Constructs a new ConnectionOpts.
             * @param [properties] Properties to set
             */
            constructor(properties?: common.MysqlDesc.IConnectionOpts);

            /** ConnectionOpts host. */
            public host: string;

            /** ConnectionOpts username. */
            public username: string;

            /** ConnectionOpts password. */
            public password: string;

            /** ConnectionOpts database. */
            public database: string;

            /**
             * Creates a new ConnectionOpts instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ConnectionOpts instance
             */
            public static create(properties?: common.MysqlDesc.IConnectionOpts): common.MysqlDesc.ConnectionOpts;

            /**
             * Encodes the specified ConnectionOpts message. Does not implicitly {@link common.MysqlDesc.ConnectionOpts.verify|verify} messages.
             * @param message ConnectionOpts message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: common.MysqlDesc.IConnectionOpts, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ConnectionOpts message, length delimited. Does not implicitly {@link common.MysqlDesc.ConnectionOpts.verify|verify} messages.
             * @param message ConnectionOpts message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: common.MysqlDesc.IConnectionOpts, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a ConnectionOpts message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns ConnectionOpts
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.MysqlDesc.ConnectionOpts;

            /**
             * Decodes a ConnectionOpts message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns ConnectionOpts
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.MysqlDesc.ConnectionOpts;

            /**
             * Verifies a ConnectionOpts message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a ConnectionOpts message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ConnectionOpts
             */
            public static fromObject(object: { [k: string]: any }): common.MysqlDesc.ConnectionOpts;

            /**
             * Creates a plain object from a ConnectionOpts message. Also converts values to other types if specified.
             * @param message ConnectionOpts
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: common.MysqlDesc.ConnectionOpts, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ConnectionOpts to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for ConnectionOpts
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a Statement. */
        interface IStatement {

            /** Statement statement */
            statement?: (string|null);

            /** Statement extractors */
            extractors?: (common.MysqlDesc.Statement.IExtractor[]|null);
        }

        /** Represents a Statement. */
        class Statement implements IStatement {

            /**
             * Constructs a new Statement.
             * @param [properties] Properties to set
             */
            constructor(properties?: common.MysqlDesc.IStatement);

            /** Statement statement. */
            public statement: string;

            /** Statement extractors. */
            public extractors: common.MysqlDesc.Statement.IExtractor[];

            /**
             * Creates a new Statement instance using the specified properties.
             * @param [properties] Properties to set
             * @returns Statement instance
             */
            public static create(properties?: common.MysqlDesc.IStatement): common.MysqlDesc.Statement;

            /**
             * Encodes the specified Statement message. Does not implicitly {@link common.MysqlDesc.Statement.verify|verify} messages.
             * @param message Statement message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: common.MysqlDesc.IStatement, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified Statement message, length delimited. Does not implicitly {@link common.MysqlDesc.Statement.verify|verify} messages.
             * @param message Statement message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: common.MysqlDesc.IStatement, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a Statement message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns Statement
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.MysqlDesc.Statement;

            /**
             * Decodes a Statement message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns Statement
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.MysqlDesc.Statement;

            /**
             * Verifies a Statement message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a Statement message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns Statement
             */
            public static fromObject(object: { [k: string]: any }): common.MysqlDesc.Statement;

            /**
             * Creates a plain object from a Statement message. Also converts values to other types if specified.
             * @param message Statement
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: common.MysqlDesc.Statement, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this Statement to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for Statement
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        namespace Statement {

            /** Properties of an Extractor. */
            interface IExtractor {

                /** Extractor index */
                index?: (number|null);

                /** Extractor extractor */
                extractor?: (string|null);
            }

            /** Represents an Extractor. */
            class Extractor implements IExtractor {

                /**
                 * Constructs a new Extractor.
                 * @param [properties] Properties to set
                 */
                constructor(properties?: common.MysqlDesc.Statement.IExtractor);

                /** Extractor index. */
                public index: number;

                /** Extractor extractor. */
                public extractor: string;

                /**
                 * Creates a new Extractor instance using the specified properties.
                 * @param [properties] Properties to set
                 * @returns Extractor instance
                 */
                public static create(properties?: common.MysqlDesc.Statement.IExtractor): common.MysqlDesc.Statement.Extractor;

                /**
                 * Encodes the specified Extractor message. Does not implicitly {@link common.MysqlDesc.Statement.Extractor.verify|verify} messages.
                 * @param message Extractor message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encode(message: common.MysqlDesc.Statement.IExtractor, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Encodes the specified Extractor message, length delimited. Does not implicitly {@link common.MysqlDesc.Statement.Extractor.verify|verify} messages.
                 * @param message Extractor message or plain object to encode
                 * @param [writer] Writer to encode to
                 * @returns Writer
                 */
                public static encodeDelimited(message: common.MysqlDesc.Statement.IExtractor, writer?: $protobuf.Writer): $protobuf.Writer;

                /**
                 * Decodes an Extractor message from the specified reader or buffer.
                 * @param reader Reader or buffer to decode from
                 * @param [length] Message length if known beforehand
                 * @returns Extractor
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.MysqlDesc.Statement.Extractor;

                /**
                 * Decodes an Extractor message from the specified reader or buffer, length delimited.
                 * @param reader Reader or buffer to decode from
                 * @returns Extractor
                 * @throws {Error} If the payload is not a reader or valid buffer
                 * @throws {$protobuf.util.ProtocolError} If required fields are missing
                 */
                public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.MysqlDesc.Statement.Extractor;

                /**
                 * Verifies an Extractor message.
                 * @param message Plain object to verify
                 * @returns `null` if valid, otherwise the reason why it is not
                 */
                public static verify(message: { [k: string]: any }): (string|null);

                /**
                 * Creates an Extractor message from a plain object. Also converts values to their respective internal types.
                 * @param object Plain object
                 * @returns Extractor
                 */
                public static fromObject(object: { [k: string]: any }): common.MysqlDesc.Statement.Extractor;

                /**
                 * Creates a plain object from an Extractor message. Also converts values to other types if specified.
                 * @param message Extractor
                 * @param [options] Conversion options
                 * @returns Plain object
                 */
                public static toObject(message: common.MysqlDesc.Statement.Extractor, options?: $protobuf.IConversionOptions): { [k: string]: any };

                /**
                 * Converts this Extractor to JSON.
                 * @returns JSON object
                 */
                public toJSON(): { [k: string]: any };

                /**
                 * Gets the default type url for Extractor
                 * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
                 * @returns The default type url
                 */
                public static getTypeUrl(typeUrlPrefix?: string): string;
            }
        }
    }

    /** Properties of a RedisDesc. */
    interface IRedisDesc {

        /** RedisDesc connectionOpts */
        connectionOpts?: (common.RedisDesc.IConnectionOpts|null);

        /** RedisDesc keyExtractor */
        keyExtractor?: (common.IFunc|null);

        /** RedisDesc valueExtractor */
        valueExtractor?: (common.IFunc|null);
    }

    /** Represents a RedisDesc. */
    class RedisDesc implements IRedisDesc {

        /**
         * Constructs a new RedisDesc.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IRedisDesc);

        /** RedisDesc connectionOpts. */
        public connectionOpts?: (common.RedisDesc.IConnectionOpts|null);

        /** RedisDesc keyExtractor. */
        public keyExtractor?: (common.IFunc|null);

        /** RedisDesc valueExtractor. */
        public valueExtractor?: (common.IFunc|null);

        /**
         * Creates a new RedisDesc instance using the specified properties.
         * @param [properties] Properties to set
         * @returns RedisDesc instance
         */
        public static create(properties?: common.IRedisDesc): common.RedisDesc;

        /**
         * Encodes the specified RedisDesc message. Does not implicitly {@link common.RedisDesc.verify|verify} messages.
         * @param message RedisDesc message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IRedisDesc, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified RedisDesc message, length delimited. Does not implicitly {@link common.RedisDesc.verify|verify} messages.
         * @param message RedisDesc message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IRedisDesc, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a RedisDesc message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns RedisDesc
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.RedisDesc;

        /**
         * Decodes a RedisDesc message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns RedisDesc
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.RedisDesc;

        /**
         * Verifies a RedisDesc message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a RedisDesc message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns RedisDesc
         */
        public static fromObject(object: { [k: string]: any }): common.RedisDesc;

        /**
         * Creates a plain object from a RedisDesc message. Also converts values to other types if specified.
         * @param message RedisDesc
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.RedisDesc, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this RedisDesc to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for RedisDesc
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    namespace RedisDesc {

        /** Properties of a ConnectionOpts. */
        interface IConnectionOpts {

            /** ConnectionOpts host */
            host?: (string|null);

            /** ConnectionOpts username */
            username?: (string|null);

            /** ConnectionOpts password */
            password?: (string|null);

            /** ConnectionOpts database */
            database?: (number|Long|null);

            /** ConnectionOpts tls */
            tls?: (boolean|null);
        }

        /** Represents a ConnectionOpts. */
        class ConnectionOpts implements IConnectionOpts {

            /**
             * Constructs a new ConnectionOpts.
             * @param [properties] Properties to set
             */
            constructor(properties?: common.RedisDesc.IConnectionOpts);

            /** ConnectionOpts host. */
            public host: string;

            /** ConnectionOpts username. */
            public username: string;

            /** ConnectionOpts password. */
            public password: string;

            /** ConnectionOpts database. */
            public database: (number|Long);

            /** ConnectionOpts tls. */
            public tls: boolean;

            /**
             * Creates a new ConnectionOpts instance using the specified properties.
             * @param [properties] Properties to set
             * @returns ConnectionOpts instance
             */
            public static create(properties?: common.RedisDesc.IConnectionOpts): common.RedisDesc.ConnectionOpts;

            /**
             * Encodes the specified ConnectionOpts message. Does not implicitly {@link common.RedisDesc.ConnectionOpts.verify|verify} messages.
             * @param message ConnectionOpts message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: common.RedisDesc.IConnectionOpts, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified ConnectionOpts message, length delimited. Does not implicitly {@link common.RedisDesc.ConnectionOpts.verify|verify} messages.
             * @param message ConnectionOpts message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: common.RedisDesc.IConnectionOpts, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a ConnectionOpts message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns ConnectionOpts
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.RedisDesc.ConnectionOpts;

            /**
             * Decodes a ConnectionOpts message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns ConnectionOpts
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.RedisDesc.ConnectionOpts;

            /**
             * Verifies a ConnectionOpts message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a ConnectionOpts message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns ConnectionOpts
             */
            public static fromObject(object: { [k: string]: any }): common.RedisDesc.ConnectionOpts;

            /**
             * Creates a plain object from a ConnectionOpts message. Also converts values to other types if specified.
             * @param message ConnectionOpts
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: common.RedisDesc.ConnectionOpts, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this ConnectionOpts to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for ConnectionOpts
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }
    }

    /** Stream Graph Status. It shows which status a stream job is now. */
    enum DataflowStatus {
        INITIALIZED = 0,
        RUNNING = 1,
        CLOSING = 2,
        CLOSED = 3
    }

    /** Properties of a Dataflow. */
    interface IDataflow {

        /** Dataflow jobId */
        jobId?: (common.IResourceId|null);

        /** Dataflow meta */
        meta?: (common.IDataflowMeta[]|null);

        /** Dataflow nodes */
        nodes?: ({ [k: string]: common.IOperatorInfo }|null);
    }

    /** Represents a Dataflow. */
    class Dataflow implements IDataflow {

        /**
         * Constructs a new Dataflow.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IDataflow);

        /** Dataflow jobId. */
        public jobId?: (common.IResourceId|null);

        /** Dataflow meta. */
        public meta: common.IDataflowMeta[];

        /** Dataflow nodes. */
        public nodes: { [k: string]: common.IOperatorInfo };

        /**
         * Creates a new Dataflow instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Dataflow instance
         */
        public static create(properties?: common.IDataflow): common.Dataflow;

        /**
         * Encodes the specified Dataflow message. Does not implicitly {@link common.Dataflow.verify|verify} messages.
         * @param message Dataflow message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IDataflow, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Dataflow message, length delimited. Does not implicitly {@link common.Dataflow.verify|verify} messages.
         * @param message Dataflow message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IDataflow, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Dataflow message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Dataflow
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Dataflow;

        /**
         * Decodes a Dataflow message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Dataflow
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Dataflow;

        /**
         * Verifies a Dataflow message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Dataflow message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Dataflow
         */
        public static fromObject(object: { [k: string]: any }): common.Dataflow;

        /**
         * Creates a plain object from a Dataflow message. Also converts values to other types if specified.
         * @param message Dataflow
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Dataflow, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Dataflow to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Dataflow
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Window. */
    interface IWindow {

        /** Window fixed */
        fixed?: (common.Window.IFixedWindow|null);

        /** Window slide */
        slide?: (common.Window.ISlidingWindow|null);

        /** Window session */
        session?: (common.Window.ISessionWindow|null);

        /** Window trigger */
        trigger?: (common.ITrigger|null);
    }

    /** Represents a Window. */
    class Window implements IWindow {

        /**
         * Constructs a new Window.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IWindow);

        /** Window fixed. */
        public fixed?: (common.Window.IFixedWindow|null);

        /** Window slide. */
        public slide?: (common.Window.ISlidingWindow|null);

        /** Window session. */
        public session?: (common.Window.ISessionWindow|null);

        /** Window trigger. */
        public trigger?: (common.ITrigger|null);

        /** Window value. */
        public value?: ("fixed"|"slide"|"session");

        /**
         * Creates a new Window instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Window instance
         */
        public static create(properties?: common.IWindow): common.Window;

        /**
         * Encodes the specified Window message. Does not implicitly {@link common.Window.verify|verify} messages.
         * @param message Window message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IWindow, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Window message, length delimited. Does not implicitly {@link common.Window.verify|verify} messages.
         * @param message Window message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IWindow, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Window message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Window
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Window;

        /**
         * Decodes a Window message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Window
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Window;

        /**
         * Verifies a Window message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Window message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Window
         */
        public static fromObject(object: { [k: string]: any }): common.Window;

        /**
         * Creates a plain object from a Window message. Also converts values to other types if specified.
         * @param message Window
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Window, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Window to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Window
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    namespace Window {

        /** Properties of a FixedWindow. */
        interface IFixedWindow {

            /** FixedWindow size */
            size?: (common.ITime|null);
        }

        /** Represents a FixedWindow. */
        class FixedWindow implements IFixedWindow {

            /**
             * Constructs a new FixedWindow.
             * @param [properties] Properties to set
             */
            constructor(properties?: common.Window.IFixedWindow);

            /** FixedWindow size. */
            public size?: (common.ITime|null);

            /**
             * Creates a new FixedWindow instance using the specified properties.
             * @param [properties] Properties to set
             * @returns FixedWindow instance
             */
            public static create(properties?: common.Window.IFixedWindow): common.Window.FixedWindow;

            /**
             * Encodes the specified FixedWindow message. Does not implicitly {@link common.Window.FixedWindow.verify|verify} messages.
             * @param message FixedWindow message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: common.Window.IFixedWindow, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified FixedWindow message, length delimited. Does not implicitly {@link common.Window.FixedWindow.verify|verify} messages.
             * @param message FixedWindow message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: common.Window.IFixedWindow, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a FixedWindow message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns FixedWindow
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Window.FixedWindow;

            /**
             * Decodes a FixedWindow message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns FixedWindow
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Window.FixedWindow;

            /**
             * Verifies a FixedWindow message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a FixedWindow message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns FixedWindow
             */
            public static fromObject(object: { [k: string]: any }): common.Window.FixedWindow;

            /**
             * Creates a plain object from a FixedWindow message. Also converts values to other types if specified.
             * @param message FixedWindow
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: common.Window.FixedWindow, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this FixedWindow to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for FixedWindow
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a SlidingWindow. */
        interface ISlidingWindow {

            /** SlidingWindow size */
            size?: (common.ITime|null);

            /** SlidingWindow period */
            period?: (common.ITime|null);
        }

        /** Represents a SlidingWindow. */
        class SlidingWindow implements ISlidingWindow {

            /**
             * Constructs a new SlidingWindow.
             * @param [properties] Properties to set
             */
            constructor(properties?: common.Window.ISlidingWindow);

            /** SlidingWindow size. */
            public size?: (common.ITime|null);

            /** SlidingWindow period. */
            public period?: (common.ITime|null);

            /**
             * Creates a new SlidingWindow instance using the specified properties.
             * @param [properties] Properties to set
             * @returns SlidingWindow instance
             */
            public static create(properties?: common.Window.ISlidingWindow): common.Window.SlidingWindow;

            /**
             * Encodes the specified SlidingWindow message. Does not implicitly {@link common.Window.SlidingWindow.verify|verify} messages.
             * @param message SlidingWindow message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: common.Window.ISlidingWindow, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified SlidingWindow message, length delimited. Does not implicitly {@link common.Window.SlidingWindow.verify|verify} messages.
             * @param message SlidingWindow message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: common.Window.ISlidingWindow, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a SlidingWindow message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns SlidingWindow
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Window.SlidingWindow;

            /**
             * Decodes a SlidingWindow message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns SlidingWindow
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Window.SlidingWindow;

            /**
             * Verifies a SlidingWindow message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a SlidingWindow message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns SlidingWindow
             */
            public static fromObject(object: { [k: string]: any }): common.Window.SlidingWindow;

            /**
             * Creates a plain object from a SlidingWindow message. Also converts values to other types if specified.
             * @param message SlidingWindow
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: common.Window.SlidingWindow, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this SlidingWindow to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for SlidingWindow
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }

        /** Properties of a SessionWindow. */
        interface ISessionWindow {

            /** SessionWindow timeout */
            timeout?: (common.ITime|null);
        }

        /** Represents a SessionWindow. */
        class SessionWindow implements ISessionWindow {

            /**
             * Constructs a new SessionWindow.
             * @param [properties] Properties to set
             */
            constructor(properties?: common.Window.ISessionWindow);

            /** SessionWindow timeout. */
            public timeout?: (common.ITime|null);

            /**
             * Creates a new SessionWindow instance using the specified properties.
             * @param [properties] Properties to set
             * @returns SessionWindow instance
             */
            public static create(properties?: common.Window.ISessionWindow): common.Window.SessionWindow;

            /**
             * Encodes the specified SessionWindow message. Does not implicitly {@link common.Window.SessionWindow.verify|verify} messages.
             * @param message SessionWindow message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: common.Window.ISessionWindow, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified SessionWindow message, length delimited. Does not implicitly {@link common.Window.SessionWindow.verify|verify} messages.
             * @param message SessionWindow message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: common.Window.ISessionWindow, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a SessionWindow message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns SessionWindow
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Window.SessionWindow;

            /**
             * Decodes a SessionWindow message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns SessionWindow
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Window.SessionWindow;

            /**
             * Verifies a SessionWindow message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a SessionWindow message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns SessionWindow
             */
            public static fromObject(object: { [k: string]: any }): common.Window.SessionWindow;

            /**
             * Creates a plain object from a SessionWindow message. Also converts values to other types if specified.
             * @param message SessionWindow
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: common.Window.SessionWindow, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this SessionWindow to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for SessionWindow
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }
    }

    /** Properties of a Trigger. */
    interface ITrigger {

        /** Trigger watermark */
        watermark?: (common.Trigger.IWatermark|null);
    }

    /** Represents a Trigger. */
    class Trigger implements ITrigger {

        /**
         * Constructs a new Trigger.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.ITrigger);

        /** Trigger watermark. */
        public watermark?: (common.Trigger.IWatermark|null);

        /** Trigger value. */
        public value?: "watermark";

        /**
         * Creates a new Trigger instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Trigger instance
         */
        public static create(properties?: common.ITrigger): common.Trigger;

        /**
         * Encodes the specified Trigger message. Does not implicitly {@link common.Trigger.verify|verify} messages.
         * @param message Trigger message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.ITrigger, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Trigger message, length delimited. Does not implicitly {@link common.Trigger.verify|verify} messages.
         * @param message Trigger message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.ITrigger, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Trigger message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Trigger
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Trigger;

        /**
         * Decodes a Trigger message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Trigger
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Trigger;

        /**
         * Verifies a Trigger message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Trigger message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Trigger
         */
        public static fromObject(object: { [k: string]: any }): common.Trigger;

        /**
         * Creates a plain object from a Trigger message. Also converts values to other types if specified.
         * @param message Trigger
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Trigger, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Trigger to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Trigger
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    namespace Trigger {

        /** Properties of a Watermark. */
        interface IWatermark {

            /** Watermark triggerTime */
            triggerTime?: (common.ITime|null);
        }

        /** Represents a Watermark. */
        class Watermark implements IWatermark {

            /**
             * Constructs a new Watermark.
             * @param [properties] Properties to set
             */
            constructor(properties?: common.Trigger.IWatermark);

            /** Watermark triggerTime. */
            public triggerTime?: (common.ITime|null);

            /**
             * Creates a new Watermark instance using the specified properties.
             * @param [properties] Properties to set
             * @returns Watermark instance
             */
            public static create(properties?: common.Trigger.IWatermark): common.Trigger.Watermark;

            /**
             * Encodes the specified Watermark message. Does not implicitly {@link common.Trigger.Watermark.verify|verify} messages.
             * @param message Watermark message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encode(message: common.Trigger.IWatermark, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Encodes the specified Watermark message, length delimited. Does not implicitly {@link common.Trigger.Watermark.verify|verify} messages.
             * @param message Watermark message or plain object to encode
             * @param [writer] Writer to encode to
             * @returns Writer
             */
            public static encodeDelimited(message: common.Trigger.IWatermark, writer?: $protobuf.Writer): $protobuf.Writer;

            /**
             * Decodes a Watermark message from the specified reader or buffer.
             * @param reader Reader or buffer to decode from
             * @param [length] Message length if known beforehand
             * @returns Watermark
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Trigger.Watermark;

            /**
             * Decodes a Watermark message from the specified reader or buffer, length delimited.
             * @param reader Reader or buffer to decode from
             * @returns Watermark
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Trigger.Watermark;

            /**
             * Verifies a Watermark message.
             * @param message Plain object to verify
             * @returns `null` if valid, otherwise the reason why it is not
             */
            public static verify(message: { [k: string]: any }): (string|null);

            /**
             * Creates a Watermark message from a plain object. Also converts values to their respective internal types.
             * @param object Plain object
             * @returns Watermark
             */
            public static fromObject(object: { [k: string]: any }): common.Trigger.Watermark;

            /**
             * Creates a plain object from a Watermark message. Also converts values to other types if specified.
             * @param message Watermark
             * @param [options] Conversion options
             * @returns Plain object
             */
            public static toObject(message: common.Trigger.Watermark, options?: $protobuf.IConversionOptions): { [k: string]: any };

            /**
             * Converts this Watermark to JSON.
             * @returns JSON object
             */
            public toJSON(): { [k: string]: any };

            /**
             * Gets the default type url for Watermark
             * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
             * @returns The default type url
             */
            public static getTypeUrl(typeUrlPrefix?: string): string;
        }
    }

    /** Properties of a ResourceId. */
    interface IResourceId {

        /** ResourceId resourceId */
        resourceId?: (string|null);

        /** ResourceId namespaceId */
        namespaceId?: (string|null);
    }

    /** JobId, represents a stream job. */
    class ResourceId implements IResourceId {

        /**
         * Constructs a new ResourceId.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IResourceId);

        /** ResourceId resourceId. */
        public resourceId: string;

        /** ResourceId namespaceId. */
        public namespaceId: string;

        /**
         * Creates a new ResourceId instance using the specified properties.
         * @param [properties] Properties to set
         * @returns ResourceId instance
         */
        public static create(properties?: common.IResourceId): common.ResourceId;

        /**
         * Encodes the specified ResourceId message. Does not implicitly {@link common.ResourceId.verify|verify} messages.
         * @param message ResourceId message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IResourceId, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified ResourceId message, length delimited. Does not implicitly {@link common.ResourceId.verify|verify} messages.
         * @param message ResourceId message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IResourceId, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a ResourceId message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns ResourceId
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.ResourceId;

        /**
         * Decodes a ResourceId message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns ResourceId
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.ResourceId;

        /**
         * Verifies a ResourceId message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a ResourceId message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns ResourceId
         */
        public static fromObject(object: { [k: string]: any }): common.ResourceId;

        /**
         * Creates a plain object from a ResourceId message. Also converts values to other types if specified.
         * @param message ResourceId
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.ResourceId, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this ResourceId to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for ResourceId
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a Response. */
    interface IResponse {

        /** Response status */
        status?: (string|null);

        /** Response errMsg */
        errMsg?: (string|null);
    }

    /** Represents a Response. */
    class Response implements IResponse {

        /**
         * Constructs a new Response.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IResponse);

        /** Response status. */
        public status: string;

        /** Response errMsg. */
        public errMsg: string;

        /**
         * Creates a new Response instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Response instance
         */
        public static create(properties?: common.IResponse): common.Response;

        /**
         * Encodes the specified Response message. Does not implicitly {@link common.Response.verify|verify} messages.
         * @param message Response message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Response message, length delimited. Does not implicitly {@link common.Response.verify|verify} messages.
         * @param message Response message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IResponse, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Response message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Response
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Response;

        /**
         * Decodes a Response message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Response
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Response;

        /**
         * Verifies a Response message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Response message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Response
         */
        public static fromObject(object: { [k: string]: any }): common.Response;

        /**
         * Creates a plain object from a Response message. Also converts values to other types if specified.
         * @param message Response
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Response, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Response to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Response
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** Properties of a HostAddr. */
    interface IHostAddr {

        /** HostAddr host */
        host?: (string|null);

        /** HostAddr port */
        port?: (number|null);
    }

    /** Represents a HostAddr. */
    class HostAddr implements IHostAddr {

        /**
         * Constructs a new HostAddr.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.IHostAddr);

        /** HostAddr host. */
        public host: string;

        /** HostAddr port. */
        public port: number;

        /**
         * Creates a new HostAddr instance using the specified properties.
         * @param [properties] Properties to set
         * @returns HostAddr instance
         */
        public static create(properties?: common.IHostAddr): common.HostAddr;

        /**
         * Encodes the specified HostAddr message. Does not implicitly {@link common.HostAddr.verify|verify} messages.
         * @param message HostAddr message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.IHostAddr, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified HostAddr message, length delimited. Does not implicitly {@link common.HostAddr.verify|verify} messages.
         * @param message HostAddr message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.IHostAddr, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a HostAddr message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns HostAddr
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.HostAddr;

        /**
         * Decodes a HostAddr message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns HostAddr
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.HostAddr;

        /**
         * Verifies a HostAddr message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a HostAddr message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns HostAddr
         */
        public static fromObject(object: { [k: string]: any }): common.HostAddr;

        /**
         * Creates a plain object from a HostAddr message. Also converts values to other types if specified.
         * @param message HostAddr
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.HostAddr, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this HostAddr to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for HostAddr
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** DataTypeEnum enum. */
    enum DataTypeEnum {
        DATA_TYPE_ENUM_UNSPECIFIED = 0,
        DATA_TYPE_ENUM_BIGINT = 1,
        DATA_TYPE_ENUM_NUMBER = 2,
        DATA_TYPE_ENUM_NULL = 3,
        DATA_TYPE_ENUM_STRING = 4,
        DATA_TYPE_ENUM_BOOLEAN = 5,
        DATA_TYPE_ENUM_OBJECT = 6,
        DATA_TYPE_ENUM_ARRAY = 7
    }

    /** Properties of a Time. */
    interface ITime {

        /** Time millis */
        millis?: (number|Long|null);

        /** Time seconds */
        seconds?: (number|Long|null);

        /** Time minutes */
        minutes?: (number|null);

        /** Time hours */
        hours?: (number|null);
    }

    /** Represents a Time. */
    class Time implements ITime {

        /**
         * Constructs a new Time.
         * @param [properties] Properties to set
         */
        constructor(properties?: common.ITime);

        /** Time millis. */
        public millis: (number|Long);

        /** Time seconds. */
        public seconds: (number|Long);

        /** Time minutes. */
        public minutes: number;

        /** Time hours. */
        public hours: number;

        /**
         * Creates a new Time instance using the specified properties.
         * @param [properties] Properties to set
         * @returns Time instance
         */
        public static create(properties?: common.ITime): common.Time;

        /**
         * Encodes the specified Time message. Does not implicitly {@link common.Time.verify|verify} messages.
         * @param message Time message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encode(message: common.ITime, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Encodes the specified Time message, length delimited. Does not implicitly {@link common.Time.verify|verify} messages.
         * @param message Time message or plain object to encode
         * @param [writer] Writer to encode to
         * @returns Writer
         */
        public static encodeDelimited(message: common.ITime, writer?: $protobuf.Writer): $protobuf.Writer;

        /**
         * Decodes a Time message from the specified reader or buffer.
         * @param reader Reader or buffer to decode from
         * @param [length] Message length if known beforehand
         * @returns Time
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decode(reader: ($protobuf.Reader|Uint8Array), length?: number): common.Time;

        /**
         * Decodes a Time message from the specified reader or buffer, length delimited.
         * @param reader Reader or buffer to decode from
         * @returns Time
         * @throws {Error} If the payload is not a reader or valid buffer
         * @throws {$protobuf.util.ProtocolError} If required fields are missing
         */
        public static decodeDelimited(reader: ($protobuf.Reader|Uint8Array)): common.Time;

        /**
         * Verifies a Time message.
         * @param message Plain object to verify
         * @returns `null` if valid, otherwise the reason why it is not
         */
        public static verify(message: { [k: string]: any }): (string|null);

        /**
         * Creates a Time message from a plain object. Also converts values to their respective internal types.
         * @param object Plain object
         * @returns Time
         */
        public static fromObject(object: { [k: string]: any }): common.Time;

        /**
         * Creates a plain object from a Time message. Also converts values to other types if specified.
         * @param message Time
         * @param [options] Conversion options
         * @returns Plain object
         */
        public static toObject(message: common.Time, options?: $protobuf.IConversionOptions): { [k: string]: any };

        /**
         * Converts this Time to JSON.
         * @returns JSON object
         */
        public toJSON(): { [k: string]: any };

        /**
         * Gets the default type url for Time
         * @param [typeUrlPrefix] your custom typeUrlPrefix(default "type.googleapis.com")
         * @returns The default type url
         */
        public static getTypeUrl(typeUrlPrefix?: string): string;
    }

    /** ErrorCode enum. */
    enum ErrorCode {
        ERROR_CODE_UNSPECIFIED = 0,
        ERROR_CODE_RESOURCE_NOT_FOUND = 1,
        ERROR_CODE_RPC_UNIMPLEMENTED = 2,
        ERROR_CODE_RPC_UNAVAILABLE = 3,
        ERROR_CODE_RPC_UNAUTHORIZED = 4,
        ERROR_CODE_RPC_INVALID_ARGUMENT = 5,
        ERROR_CODE_RPC_PERMISSION_DENIED = 6,
        ERROR_CODE_INTERNAL_ERROR = 7,
        ERROR_CODE_TOO_MANY_REQUEST = 8,
        ERROR_CODE_RPC_BIND_FAILED = 9,
        ERROR_CODE_GOOGLE_AUTH_FAILED = 10,
        ERROR_CODE_DATAFLOW_OPERATOR_INFO_MISSING = 11,
        ERROR_CODE_CYCLIC_DATAFLOW = 12
    }
}
