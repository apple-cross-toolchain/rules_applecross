//
// VObj1.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 11.0, iOS 14.0, tvOS 14.0, visionOS 1.0, *)
@available(watchOS, unavailable)
public class VObj1Input : MLFeatureProvider {

    /// image as color (kCVPixelFormatType_32BGRA) image buffer, 299 pixels wide by 299 pixels high
    public var image: CVPixelBuffer

    public var featureNames: Set<String> { ["image"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "image" {
            return MLFeatureValue(pixelBuffer: image)
        }
        return nil
    }

    public init(image: CVPixelBuffer) {
        self.image = image
    }

    public convenience init(imageWith image: CGImage) throws {
        self.init(image: try MLFeatureValue(cgImage: image, pixelsWide: 299, pixelsHigh: 299, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!)
    }

    public convenience init(imageAt image: URL) throws {
        self.init(image: try MLFeatureValue(imageAt: image, pixelsWide: 299, pixelsHigh: 299, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!)
    }

    public func setImage(with image: CGImage) throws  {
        self.image = try MLFeatureValue(cgImage: image, pixelsWide: 299, pixelsHigh: 299, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

    public func setImage(with image: URL) throws  {
        self.image = try MLFeatureValue(imageAt: image, pixelsWide: 299, pixelsHigh: 299, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 11.0, iOS 14.0, tvOS 14.0, visionOS 1.0, *)
@available(watchOS, unavailable)
public class VObj1Output : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// hi as 288 × 35 × 35 3-dimensional array of floats
    public var hi: MLMultiArray {
        provider.featureValue(for: "hi")!.multiArrayValue!
    }

    /// hi as 288 × 35 × 35 3-dimensional array of floats
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public var hiShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(hi)
    }

    /// lo as 768 × 17 × 17 3-dimensional array of floats
    public var lo: MLMultiArray {
        provider.featureValue(for: "lo")!.multiArrayValue!
    }

    /// lo as 768 × 17 × 17 3-dimensional array of floats
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public var loShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(lo)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(hi: MLMultiArray, lo: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["hi" : MLFeatureValue(multiArray: hi), "lo" : MLFeatureValue(multiArray: lo)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 11.0, iOS 14.0, tvOS 14.0, visionOS 1.0, *)
@available(watchOS, unavailable)
public class VObj1 {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "VObj1", withExtension:"mlmodelc")!
    }

    /**
        Construct VObj1 instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of VObj1.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `VObj1.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct a model with configuration

        - parameters:
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(configuration: MLModelConfiguration = MLModelConfiguration()) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct VObj1 instance with explicit path to mlmodelc file
        - parameters:
           - modelURL: the file url of the model

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL) throws {
        try self.init(model: MLModel(contentsOf: modelURL))
    }

    /**
        Construct a model with URL of the .mlmodelc directory and configuration

        - parameters:
           - modelURL: the file url of the model
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {
        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))
    }

    /**
        Construct VObj1 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<VObj1, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct VObj1 instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> VObj1 {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct VObj1 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<VObj1, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(VObj1(model: model)))
            }
        }
    }

    /**
        Construct VObj1 instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> VObj1 {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return VObj1(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as VObj1Input

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as VObj1Output
    */
    public func prediction(input: VObj1Input) throws -> VObj1Output {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as VObj1Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as VObj1Output
    */
    public func prediction(input: VObj1Input, options: MLPredictionOptions) throws -> VObj1Output {
        let outFeatures = try model.prediction(from: input, options: options)
        return VObj1Output(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as VObj1Input
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as VObj1Output
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: VObj1Input, options: MLPredictionOptions = MLPredictionOptions()) async throws -> VObj1Output {
        let outFeatures = try await model.prediction(from: input, options: options)
        return VObj1Output(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - image: color (kCVPixelFormatType_32BGRA) image buffer, 299 pixels wide by 299 pixels high

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as VObj1Output
    */
    public func prediction(image: CVPixelBuffer) throws -> VObj1Output {
        let input_ = VObj1Input(image: image)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [VObj1Input]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [VObj1Output]
    */
    public func predictions(inputs: [VObj1Input], options: MLPredictionOptions = MLPredictionOptions()) throws -> [VObj1Output] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [VObj1Output] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  VObj1Output(features: outProvider)
            results.append(result)
        }
        return results
    }
}
