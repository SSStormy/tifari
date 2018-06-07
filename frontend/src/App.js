import React, { Component } from 'react';
import './App.css';
import SearchField from './SearchField.js';
import ImageSlot from './ImageSlot.js';
import ImageEditor from './ImageEditor.js';
import TagList from './TagList.js';
import TifariAPI from "./APIComms.js";
import {ldebug, assert} from "./Logging.js";

class StateMutator {
    constructor(app, oldState) {
        this.newState = {};

        this.oldState = oldState;
        this.app = app;
    }

    getFinalState() {
        return this.newState;
    }

    getOldState() {
        return this.oldState;
    }

    renderImageList() {
        ldebug("Rendering image list");

        let images = this.getQueriedImagesAndMarkMutated();

        let mappedImages = images.map(img => 
            <ImageSlot 
                img={img} 
                key={img.id} 
                onClick={() => this.app.onSelectImage(img)}
            />
        );

        this.newState.queriedImagesAsElements = mappedImages;

        return this;
    }

    setImageList(images) {
        ldebug("Setting image list.");
        ldebug(images);

        this.newState.queriedImages = images;
        return this;
    }

    addImageToList(image) {
        ldebug("Adding image to list");
        ldebug(image);

        let images = this.getQueriedImagesAndMarkMutated();

        // avoid duplicate images
        if(images.findIndex(i => i.id === image.id) !== -1)
            return;
    
        images.push(image);

        return this;
    }

    // TODO: this might be a no-op
    markSelectedImagesAsDirty() {
        if(!this.newState.hasOwnProperty("selectedImages")) {
            ldebug("Marked selected images as dirty.");
            this.newState.selectedImages = this.oldState.selectedImages;
        }

        return this;
    }

    getQueriedImagesAndMarkMutated() {
        if(!this.newState.hasOwnProperty("queriedImages")) {
            // if we copy this, we're going to have to add a second layer of
            // indirection by making this queriedImage array store
            // references to cached values within a cachce area somewhere
            this.newState.queriedImages = this.oldState.queriedImages;
        }

        return this.newState.queriedImages;
    }

    // doesn't update the image list.
    removeImageFromList(image) {
        ldebug("Removing image from list.");
        ldebug(image);

        let images = this.getQueriedImagesAndMarkMutated();
    
        let imgIndex = images.findIndex(i => i.id === image.id);
        if(imgIndex === -1) return this;

        images.splice(imgIndex, 1);

        return this;
    }

    // doesn't update the image list.
    removeTagFromImage(image, tag) {
        ldebug("Removing tag from image");
        ldebug(image);
        ldebug(tag);

        let images = this.getQueriedImagesAndMarkMutated();

        let imgIndex = images.findIndex(i => i.id === image.id);
        if(imgIndex === -1) return this;

        let cachedImg = images[imgIndex];
        assert(cachedImg === image);

        let tagIndex = cachedImg.tags.findIndex(t => t.id === tag.id);
        if(tagIndex === -1) return this;

        cachedImg.tags.splice(tagIndex, 1);

        return this;
    }

    // doesn't update the image list.
    addTagToImage(image, tag) {
        ldebug("Adding tag to image");
        ldebug(image);
        ldebug(tag);

        let images = this.getQueriedImagesAndMarkMutated();

        let imgIndex = images.findIndex(i => i.id === image.id);
        if(imgIndex === -1) return this;

        let cachedImg = images[imgIndex];

        let tagIndex = cachedImg.tags.findIndex(t => t.id === tag.id);

        if(tagIndex === -1) {
            cachedImg.tags.push(tag);
        }

        return this;
    }

    addSelectedImage(image) {
        ldebug("Adding selected image");
        ldebug(image);

        this.markSelectedImagesAsDirty();

        // avoid duplicate images
        if(this.newState.selectedImages.findIndex(i => i.id === image.id) !== -1)
            return;

        this.newState.selectedImages.push(image);
        return this;
    }

    clearSelectedImages() {
        ldebug("Clearing selected images");

        this.newState.selectedImages = [];
        return this;
    }

    setSearchTags(tagsArray) {
        ldebug("Setting search tags");
        ldebug(tagsArray);

        this.newState.searchTagNames = tagsArray;
        return this;
    }

    setIsInToBeTaggedList(state) {
        ldebug("Setting is in to be tagged list state to");
        ldebug(state);

        this.newState.isInToTagList = state;
        return this;
    }

    setTagListDisplayState(state) {
        ldebug("Setting tag list display state to");
        ldebug(state);

        this.newState.displayTagList = state;
        return this;
    }

    setToBeTaggedListSize(size) {
        ldebug("Seting to be tagged list size to");
        ldebug(size);

        this.newState.tagQueueSize = size;
        return this;
    }
}

class App extends Component {

    constructor(props) {
        super(props);

        this.state = {
            queriedImages: [],
            queriedImagesAsElements: [],
            isInToTagList: false,
            selectedImages: [],
            displayTagList: false,
            tagQueueSize: 0,
            searchTagNames: [],
        };

        this.foreignOnSearch                = this.foreignOnSearch.bind(this);
        this.foreignToggleTagListDisplay    = this.foreignToggleTagListDisplay.bind(this);
        this.foreignEscKeyListener          = this.foreignEscKeyListener.bind(this);
        this.foreignViewToBeTaggedList      = this.foreignViewToBeTaggedList.bind(this);

        this.foreignOnEditorRemoveTagFromSelected = this.foreignOnEditorRemoveTagFromSelected.bind(this);
        this.foreignOnEditorAddTagToSelected = this.foreignOnEditorAddTagToSelected.bind(this);
    }

    hideEditorSidebar() {
        this.mutateState(mut => mut.clearSelectedImages());
    }

    foreignEscKeyListener(ev) {
        if(ev.key === "Escape") {
            this.hideEditorSidebar();
        }
    }

    componentWillMount() {
        this.updateToBeTaggedListSize()
    }

    componentDidMount() {
        document.addEventListener("keypress", this.foreignEscKeyListener, false);
    }

    componentWillUnmount() {
        document.removeEventListener("keypress", this.foreignEscKeyListener, false);
    }

    mutateState(lambda) {
        this.setState(oldState => {
            let mutator = new StateMutator(this, oldState);
            lambda(mutator);
            return mutator.getFinalState();
        });
    }

    // callback that's called when we want to select an image
    onSelectImage(img) {
        this.mutateState(mut => {
            mut.addSelectedImage(img);
        })
    }

    // callback that's called when we want to search the backend for tags
    foreignOnSearch(query) {
        let tags = query.split(" ");

        TifariAPI.search(tags)
            .then(images =>
                this.mutateState(mut =>
                    mut.setSearchTags(tags)
                       .setImageList(images)
                       .renderImageList()
                       .setIsInToBeTaggedList(false))
                );
    }

    updateToBeTaggedListSize() {
        TifariAPI.getTagQueueSize()
            .then(size => 
                this.mutateState(mut => mut.setToBeTaggedListSize(size)));
    }

    doesSearchContainTags(tags) {
        const intersection = this.state.searchTagNames.filter(
            tagName => tags.findIndex(tag => tag.name === tagName) !== -1);

        return 0 >= intersection.length;
    }

    // callback that's called when we remove a tag from an image
    foreignOnEditorRemoveTagFromSelected(tag) {

        let imageIds = this.state.selectedImages.map(img => img.id);

        TifariAPI.removeTags([tag.id], imageIds);

        this.mutateState(mut => {

            let rerender = false;
            mut.getOldState().selectedImages
                .forEach(image => {

                    // TODO : @BUG
                    // Removing a tag doesn't seem to actually be working.
                    // Repro:
                    // go to the to-tag list, add a tag to an image, remove that tag.
                    // the image will not be readded to the list and the tag will still be there.
                    
                    mut.removeTagFromImage(image, tag);

                    if(mut.getOldState().isInToTagList && 0 >= image.tags.length) {
                        mut.addImageToList(image);
                    } else if(this.doesSearchContainTags(image.tags)) {
                        mut.removeImageFromList(image);
                        rerender = true;
                    }

                });

            // TODO : update tag list
            mut.markSelectedImagesAsDirty();
            if(rerender);
               mut.renderImageList();
        });

        this.updateToBeTaggedListSize();
    }

    // callback that's called whenever we add a tag to an image
    foreignOnEditorAddTagToSelected(tagNames) { 

        let imageIds = this.state.selectedImages.map(img => img.id);

        TifariAPI.addTags(tagNames, imageIds)
            .then(tags => this.mutateState(mut => {

                // add each tag to each image
                tags.forEach(
                    tag => mut.getOldState().selectedImages.forEach(
                        img => mut.addTagToImage(img, tag))
                );

                let rerender = false;

                // if we're in the to tag list, remove all the selected images
                // since we just added tags to those images.
                if(mut.getOldState().isInToTagList) {
                    mut.getOldState().selectedImages
                        .forEach(img => mut.removeImageFromList(img));
                    rerender = true;
                } else {
                    mut.getOldState().selectedImages.forEach(img =>{
                        if(this.doesSearchContainTags(img.tags)) {
                            mut.addImageToList(img);
                            rerender = true;
                        }
                    });
                }
                    
                mut.markSelectedImagesAsDirty();

                if(rerender)
                    mut.renderImageList();
            })
        );

        // TODO : update tag list
        this.updateToBeTaggedListSize();
    }

    foreignToggleTagListDisplay() {
        this.mutateState(mut => mut.setTagListDisplayState(!mut.getOldState().displayTagList));
    }

    foreignViewToBeTaggedList() {
        TifariAPI.getToBeTaggedList()
            .then(images =>
                this.mutateState(mut => 
                    mut.setImageList(images)
                        .renderImageList()
                        .setIsInToBeTaggedList(true))
            );
    }

    render() {
        return (
            <div className="App">
                {this.state.selectedImages.length > 0 &&
                    <ImageEditor 
                        images={this.state.selectedImages}
                        onAddTag={this.foreignOnEditorAddTagToSelected}
                        onRemoveTag={this.foreignOnEditorRemoveTagFromSelected}
                    />
                }

                {this.state.displayTagList &&
                    <TagList
                    />
                }

                <header>

                    <button onClick={this.foreignViewToBeTaggedList}>
                        View To-Tag List({this.state.tagQueueSize})
                    </button>

                    <button onClick={() => TifariAPI.reloadRoot()}>
                        Reload images
                    </button>
            
                    <button onClick={this.foreignToggleTagListDisplay}>
                        {this.state.displayTagList ? "Hide" : "Show"} tag list
                    </button>

                    <SearchField onChange = {this.foreignOnSearch} />
                </header>

                <ul>{this.state.queriedImagesAsElements}</ul>
            </div>
        );
    }
}

export default App;
