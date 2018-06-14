import React, { Component } from 'react';
import './App.css';

import TifariAPI from "./APIComms.js";
import { ldebug } from "./Logging.js";

import Collapse from '@material-ui/core/Collapse';
import ListItem from '@material-ui/core/ListItem';
import List from '@material-ui/core/List';
import ListItemIcon from '@material-ui/core/ListItemIcon';
import ListItemText from '@material-ui/core/ListItemText';
import CssBaseline from '@material-ui/core/CssBaseline';
import IconButton from '@material-ui/core/IconButton';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import Icon from '@material-ui/core/Icon';
import Snackbar from '@material-ui/core/Snackbar';
import Grid from '@material-ui/core/Grid';
import Button from '@material-ui/core/Button';
import TextField from '@material-ui/core/TextField';
import Card from '@material-ui/core/Card';
import Paper from '@material-ui/core/Paper';
import Drawer from '@material-ui/core/Drawer';
import Divider from '@material-ui/core/Divider';
import Chip from '@material-ui/core/Chip';
import Dialog from '@material-ui/core/Dialog';
import DialogActions from '@material-ui/core/DialogActions';
import DialogContent from '@material-ui/core/DialogContent';
import DialogContentText from '@material-ui/core/DialogContentText';
import DialogTitle from '@material-ui/core/DialogTitle';
import MenuItem from '@material-ui/core/MenuItem';
import Select from '@material-ui/core/Select';
import ListItemSecondaryAction from '@material-ui/core/ListItemSecondaryAction';
import ListItemAvatar from '@material-ui/core/ListItemAvatar';
import Avatar from '@material-ui/core/Avatar';
import SvgIcon from '@material-ui/core/SvgIcon';
import ChipInput from 'material-ui-chip-input'

const VERSION = "1.0.0";

const allOrderings = [
    {
        id: 0,
        display: "Times used, descending",
        order: function(tags) {
            tags.sort((a, b) => a.times_used < b.times_used);
        }
    },

    {
        id: 1,
        display: "Times used, ascending",
        order: function(tags) {
            tags.sort((a, b) => a.times_used > b.times_used);
        }
    },

    {
        id: 2,
        display: "Alphabetical, descending",
        order: function(tags) {
            tags.sort((a, b) => a.name < b.name);
        }
    },

    {
        id: 3,
        display: "Alphabetical, ascending",
        order: function(tags) {
            tags.sort((a, b) => a.name > b.name);
        }
    }
];

class TagList extends Component {

    constructor(props) {
        super(props);

        this.state = {
            textField: "",
        };
    }

    submitTabInput() {
        this.props.onAdd(this.state.textField);
        this.setState({textField: ""});
    }

    render() {

        const tagList = this.props.tags.map(tag => 
            <Chip 
                key={tag.id} 
                label={tag.name}
                onDelete={() => this.props.onRemove(tag)}
            />
        );

        const className = this.props.className ? this.props.className : "";

        return (
            <div className={`${className} tag-list`}>
                <span className="chip-list">
                    {tagList}
                </span>

                <span className="input-field">
                    <TextField
                        value={this.state.textField}
                        id="tag-input"
                        label="Add tags"
                        style={{paddingRight: 8}}
                        onChange={(e) => this.setState({textField: e.target.value})}
                        type="text"
                    />
                </span>

                <Button 
                    variant="fab" 
                    className="add-button"
                    onClick={() => this.submitTabInput()}
                    >
                    <Icon>add</Icon>
                </Button>
            </div>
        );
    }
}

class StateMutator {
    constructor(app, oldState) {
        this.newState = {};

        this.oldState = oldState;
        this.app = app;
    }

    setBackendUrlBuffer(val) {
        this.newState.backendUrlBuffer = val;
        return this;
    }

    getFinalState() {
        return this.newState;
    }

    getOldState() {
        return this.oldState;
    }

    setTags(tags) {
        ldebug("Setting tags list to");
        ldebug(tags);

        this.newState.tags = tags;
        return this;
    }

    setTagOrdering(orderingId) {
        this.newState.tagOrdering = allOrderings[orderingId];
        return this;
    }

    setApiState(state) {
        this.newState.apiConnected = state;
        return this;
    }

    showSnackbar(msg) {
        this.newState.showSnackbar = true;
        this.newState.snackbarMessage = msg;

        return this;
    }

    hideSnackbar() {
        this.newState.showSnackbar = false;

        return this;
    }

    orderTags() {
        
        if(!this.newState.hasOwnProperty("tags")) {
            this.newState.tags = this.oldState.tags;
        }

        let tagOrdering = this.newState.hasOwnProperty("tagOrdering")
            ? this.newState.tagOrdering
            : this.oldState.tagOrdering;

        tagOrdering.order(this.newState.tags);
    }

    getProp(name) {
        return this.newState.hasOwnProperty(name)
                ? this.newState[name]
                : this.oldState[name];
    }

    getPropMarkDirty(name) {
        if(!this.newState.hasOwnProperty(name))
            this.newState[name] = this.oldState[name];

        return this.newState[name];
    }

    setActiveImageList(imgsEnum) {
        ldebug("Setting active image list to");
        ldebug(imgsEnum);
        this.newState.activeImageListEnum = imgsEnum;
        return this;
    }

    setSelectedImages(images) {
        this.newState.selectedImages = {};
        this.newState.selectedImages.page = 0;
        this.newState.selectedImages.arr = images;
        return this;
    }

    setSearchImages(images) {
        this.newState.searchImages= {};
        this.newState.searchImages.page = 0;
        this.newState.searchImages.arr = images;
        return this;
    }

    setToBeTaggedImages(images) {
        this.newState.toBeTaggedImages= {};
        this.newState.toBeTaggedImages.page = 0;
        this.newState.toBeTaggedImages.arr = images;
        return this;
    }

    clearImgList(imgEnum) {
        this.newState[imgEnum.prop] = {};
        this.newState[imgEnum.prop].arr = [];
        this.newState[imgEnum.prop].page = 0;
        return this;
    }

    setPage(page) {
        if(1 > page) 
            page = 1;

        let images = this.getPropMarkDirty(this.getProp("activeImageListEnum").prop);
        images.page = page;
        return this;
    }

    addImageToList(imgsEnum, image) {
        let images = this.getPropMarkDirty(imgsEnum.prop).arr;

        // avoid duplicate images
        if(images.findIndex(i => i.id === image.id) !== -1)
            return this;
    
        images.push(image);

        return this;
    }

    // doesn't update the image list.
    removeImageFromList(imgsEnum, image) {
        let images = this.getPropMarkDirty(imgsEnum.prop).arr;
    
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

        let tagIndex = image.tags.findIndex(t => t.id === tag.id);
        if(tagIndex === -1) return this;

        image.tags.splice(tagIndex, 1);

        return this;
    }

    // doesn't update the image list.
    // image must be a part of the app state
    addTagToImage(image, tag) {
        ldebug("Adding tag to image");
        ldebug(image);
        ldebug(tag);

        let tagIndex = image.tags.findIndex(t => t.id === tag.id);

        if(tagIndex === -1) {
            image.tags.push(tag);
        }

        return this;
    }

    addAllInCurrentViewToSelection() {
        let viewType = this.getProp("activeImageListEnum");

        if(viewType.id === IMGS_SELECTED.id)
            return;

        let sourceImages = this.getPropMarkDirty(viewType.prop).arr;
        let targetImages = this.getPropMarkDirty(IMGS_SELECTED.prop).arr;

        let existingIds = new Set(targetImages.map(i => i.id));

        sourceImages.forEach(img => {
            if(existingIds.has(img.id))
                return;

            targetImages.push(img);
        });

        return this;
    }

    setSliderCardSize(size) {
        this.newState.sliderCardSize = size;
        return this;
    }

    setDialogBackendUrlState(state) {
        this.newState.dialogBackendUrl = state;
        return this;
    }

    setDialogAboutState(state) {
        this.newState.dialogAbout = state;
        return this;
    }

    setDialogImageRowsState(state) {
        this.newState.dialogImageRowsOpen = state;
        return this;
    }

    setTabState(state) {
        this.newState.tabState = state;

        return this;
    }
    
    appendArray(arrayName, val) {
        this.getPropMarkDirty(arrayName).push(val);
        return this;
    }

    removeFromArray(arrayName, val) {
        let arr = this.getPropMarkDirty(arrayName);
        let idx = arr.findIndex(i => i === val);
        if(idx === -1) return this;

        arr.splice(idx, 1);

        return this;
    }

    removeFromArrayByIdx(arrayName, idx) {
        let arr = this.getPropMarkDirty(arrayName);
        arr.splice(idx, 1);

        return this;
    }

    setSearchTagNames(tagNames) {
        ldebug("Setting search tags");
        ldebug(tagNames);
        this.newState.searchTagNames = tagNames;
        return this;
    }

    setDrawerOpenState(state) {
        this.newState.isDrawerOpen = state;
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

const IMGS_SEARCH   = {id: 0, prop: "searchImages" }
const IMGS_TO_TAG   = {id: 1, prop: "toBeTaggedImages" }
const IMGS_SELECTED = {id: 2, prop: "selectedImages" }

const TABS_SEARCH = 0;
const TABS_TO_TAG = 1;
const TABS_SELECTED = 2;

class App extends Component {

    constructor(props) {
        super(props);

        this.foreignShowSearchTab = this.foreignShowSearchTab.bind(this);
        this.foreignShowToBeTaggedTab = this.foreignShowToBeTaggedTab.bind(this);
        this.foreignShowSelectedTab = this.foreignShowSelectedTab.bind(this);
        this.foreignAddTagToSearch = this.foreignAddTagToSearch.bind(this);
        this.foreignRemoveTagFromSearch= this.foreignRemoveTagFromSearch.bind(this);
        this.foreignAddTagButton = this.foreignAddTagButton.bind(this);
        this.foreignRemoveTagButton= this.foreignRemoveTagButton.bind(this);

        this.state = {
            activeImageListEnum: IMGS_SEARCH,
            searchImages: { page: 0, arr: []},
            selectedImages: {page: 0, arr: []},
            toBeTaggedImages: {page: 0, arr: []},

            sliderCardSize: 2,
            backendUrlBuffer: "",
            isDrawerOpen: false,
            displayTagList: false,
            dialogImageRowsOpen: false,
            dialogAbout: false,
            dialogBackendUrl: false,
            tagQueueSize: 0,
            searchTagNames: [],
            tags: [],
            tagOrdering: allOrderings[0],
            tabState: 0,
            api: new TifariAPI("http://localhost:8001", () => this.apiSuccess(), () => this.apiError()),
            apiConnected: null,
        };
    }

    componentWillMount() {
        this.updateToBeTaggedListSize();
    }

    apiSuccess() {
        
        if(this.state.apiConnected !== null && this.state.apiConnected)
            return;

        this.updateToBeTaggedListSize();
        this.updateTagList();
        this.updateToBeTaggedList();

        this.mutateState(mut => 
            mut.setApiState(true)
        );
    }

    apiError() {
        if(this.state.apiConnected !== null && !this.state.apiConnected)
            return;

        console.log("error");
        this.mutateState(mut => 
            mut.setApiState(false)
               .showSnackbar("Failed to connect to backend.")
        );
    }

    updateToBeTaggedList() {
        this.state.api.getToBeTaggedList()
            .then(images =>
                this.mutateState(mut => mut.setToBeTaggedImages(images))
            );
    }

    mutateState(lambda) {
        this.setState(oldState => {
            let mutator = new StateMutator(this, oldState);
            lambda(mutator);
            return mutator.getFinalState();
        });
    }

    selectImage(img) {
        this.mutateState(mut => {
            mut.addImageToList(IMGS_SELECTED, img);
        })
    }

    // callback that's called when we want to search the backend for tags
    doImageSearch() {
        ldebug(this.state.searchTagNames);
        this.state.api.search(this.state.searchTagNames)
            .then(images =>
                this.mutateState(mut => mut.setSearchImages(images))
            );
    }

    foreignShowSearchTab() { 
        this.mutateState(mut => mut.setActiveImageList(IMGS_SEARCH));
    }

    foreignShowToBeTaggedTab() { 
        this.mutateState(mut => mut.setActiveImageList(IMGS_TO_TAG));
    }

    foreignShowSelectedTab() { 
        this.mutateState(mut => mut.setActiveImageList(IMGS_SELECTED));
    }

    updateToBeTaggedListSize() {
        this.state.api.getTagQueueSize().then(size => this.mutateState(mut => mut.setToBeTaggedListSize(size)));
    }

    updateTagList() {
        this.state.api.getAllTags().then(tags => { 
            tags.sort((a, b) => a.times_used < b.times_used);
            this.mutateState(mut => 
                mut.setTags(tags)
                   .orderTags()
            )
        });
    }

    doTagsMatchSearch(tags) {
        let searchTagNames= new Set(this.state.searchTagNames);
        
        if(searchTagNames.size <= 0)
            return false;

        let imageTags = new Set(tags.map(t => t.name));
    
        for(let item of searchTagNames) {
            if(!imageTags.has(item))
                return false;
        }

        return true;
    }

    // callback that's called when we remove a tag from an image
    removeTagFromSelected(tag) {

        let imageIds = this.state.selectedImages.arr.map(img => img.id);

        this.state.api.removeTags([tag.id], imageIds);

        this.mutateState(mut => {

            mut.getOldState().selectedImages.arr
                .forEach(image => this.localBookkeepTagRemoval(mut, image, tag));
        });

        this.updateTagList();
        this.updateToBeTaggedListSize();
    }
    
    // callback that's called whenever we add a tag to an image
    addTagsToSelected(tagString) { 
        tagString = tagString.trim();
        if(tagString.lengt <= 0) return;
        let tagNames = tagString.split(" ");
        let imageIds = this.state.selectedImages.arr.map(img => img.id);

        this.state.api.addTags(tagNames, imageIds)
            .then(tags => this.mutateState(mut => 
                this.localBookkeepTagsAdd(mut, mut.getOldState().selectedImages.arr, tags)
            )
        );

        this.updateTagList();
        this.updateToBeTaggedListSize();
    }

    foreignAddTagToSearch(tagName) {

        if(this.state.searchTagNames.includes(tagName))
            return;

        this.mutateState(mut => { 
            mut.appendArray("searchTagNames", tagName);
            this.doImageSearch();
        });
    }

    foreignRemoveTagFromSearch(tagName, idx) {
        this.mutateState(mut => { 
            mut.removeFromArrayByIdx("searchTagNames", idx)
            this.doImageSearch();
        });
    }

    unselectImage(image) {
        this.mutateState(mut => { 
            mut.removeImageFromList(IMGS_SELECTED, image)
            if(mut.getFinalState()[IMGS_SELECTED.prop].arr.length <= 0 
                && mut.getOldState().tabState === TABS_SELECTED) {
                mut.setTabState(TABS_SEARCH);
            }
        });
    }

    isImageSelected(image) {
        return -1 !== this.state.selectedImages.arr.findIndex(i => i.id === image.id);
    }

    isViewingSelectedImages() {
        return this.state.activeImageListEnum.id === IMGS_SELECTED.id;
    }
    
    removeTagFrom(image, tag) {
        this.state.api.removeTags([tag.id], [image.id]);

        this.mutateState(mut => {
            this.localBookkeepTagRemoval(mut, image, tag);
        });

        this.updateTagList();
        this.updateToBeTaggedListSize();

    }

    addTagsTo(image, tagString) {
        tagString = tagString.trim();
        if(tagString.length <= 0) return;

        let tagNames = tagString.split(" ");

        this.state.api.addTags(tagNames, [image.id])
            .then(tags => this.mutateState(mut => 
                this.localBookkeepTagsAdd(mut, [image], tags)
            )
        );

        this.updateTagList();
        this.updateToBeTaggedListSize();

    }

    localBookkeepTagsAdd(mut, images, tags) {

        // add each tag to each image
        tags.forEach(tag => images.forEach(
                img => mut.addTagToImage(img, tag))
        );

        images.forEach(img => { 
            if(img.tags.length > 0)
                mut.removeImageFromList(IMGS_TO_TAG, img);

            if(this.doTagsMatchSearch(img.tags)) {
                mut.addImageToList(IMGS_SEARCH, img);
            }
        });
    }

    localBookkeepTagRemoval(mut, image, tag) {
        mut.removeTagFromImage(image, tag);

        if(0 >= image.tags.length) {
            mut.addImageToList(IMGS_TO_TAG, image);
            mut.removeImageFromList(IMGS_SEARCH, image);
        } else if(!this.doTagsMatchSearch(image.tags)) {
            mut.removeImageFromList(IMGS_SEARCH, image);
        }
    }

    onClickedImageCard(ev, img, isSel) {
        if(ev.target.id === "card-image") {
            if(isSel)
                this.unselectImage(img);
            else
                this.selectImage(img)
        }
    }

    setDialogImageRowsState(state) {
        this.mutateState(mut => mut.setDialogImageRowsState(state));
    }

    setDialogBackendUrlState(state) {
        this.mutateState(mut => {
            mut.setBackendUrlBuffer(this.state.api.getEndpoint());
            mut.setDialogBackendUrlState(state);
        });
    }

    foreignAddTagButton(ev) {
        this.foreignAddTagToSearch(ev.target.value);
    }

    foreignRemoveTagButton(ev) {
        let val = ev.target.value;
        this.mutateState(mut => { 
            mut.removeFromArray("searchTagNames", val);
            this.doImageSearch();
        });
    }

    setDialogAboutState(state) { 
        this.mutateState(mut => mut.setDialogAboutState(state));
    }

    swapBackendUrl() {
        this.state.api.setEndpoint(this.state.backendUrlBuffer);
    }

    render() {
        const min = (a,b) => a > b ? b : a;

        const activeImageList = this.state[this.state.activeImageListEnum.prop];
        const imgsPerPage = 20;

        const startIdx = activeImageList.page * imgsPerPage; 
        const endIdx = min(startIdx + imgsPerPage, activeImageList.arr.length);
        const numPages = activeImageList.arr.length / imgsPerPage;

        let imageList = [];

        let nextPageButtons = [];

        for(let i = 0; i < numPages; i++) {
            // TODO : make the buttons pretty
            nextPageButtons.push(
                <Button 
                    key={i}
                    onClick={() => this.mutateState(mut => mut.setPage(i))}
                    >
                    {(i + 1).toString()}
                </Button>
            );
        }

        for(let i = startIdx;
            i < endIdx;
            i++) {

            const img = activeImageList.arr[i];
            const isSelected = this.isImageSelected(img);
            const drawSelectedMods = isSelected && !this.isViewingSelectedImages();

            imageList.push(
                <Grid item xs={12 / this.state.sliderCardSize} key={img.id}>

                    <Card 
                        square={true} 
                        elevation={5} 
                        className="card show-when-hovering"
                        onClick={(ev) => this.onClickedImageCard(ev, img, isSelected)}
                        >

                        <img 
                            alt={img.path}
                            style={{opacity: drawSelectedMods ? 0.5 : 1}}
                            id="card-image"
                            src={this.state.api.getImageUrl(img)}
                            title={img.path}
                        />

                        { drawSelectedMods &&
                        <Icon 
                            className="checkmark" 
                            style={{fontSize: 48}}
                            >
                            done_outline
                        </Icon>
                        }

                        <div className="bottom-bar show-when-hovering--on">
                            <Paper square={true}>
                                <TagList 
                                    tags={img.tags} 
                                    onAdd={(tagString) => this.addTagsTo(img, tagString)}
                                    onRemove={(tag) => this.removeTagFrom(img, tag)}
                                />
                            </Paper>
                        </div>
                        
                    </Card>


                </Grid>
            );
        }

        let selectedImageTags = [];
        if(this.state.tabState === TABS_SELECTED) {
            let existingTagIds = new Set();

            this.state.selectedImages.arr.forEach(img => {
                img.tags.forEach(tag => {
                    if(existingTagIds.has(tag.id))
                        return;

                    existingTagIds.add(tag.id);
                    selectedImageTags.push(tag);
                })
            })
        }

        return (
            <React.Fragment>
            <CssBaseline/>
                
                <Paper className="top-bar">

                    <IconButton 
                        className="icon-button"
                        onClick={() => this.mutateState(mut => mut.setDrawerOpenState(true))}
                        >
                        <Icon>menu</Icon>
                    </IconButton>

                    <Tabs className="center-field"
                        value={this.state.tabState} 
                        onChange={(e, v) => this.mutateState(mut => mut.setTabState(v))}
                        >

                        <Tab
                            label="Search" 
                            onClick={this.foreignShowSearchTab}
                        />
                        
                        { this.state.tagQueueSize > 0 &&
                        <Tab 
                            label={`To-be tagged (${this.state.tagQueueSize})`} 
                            onClick={this.foreignShowToBeTaggedTab}
                        />
                        }
                        
                        { this.state.selectedImages.arr.length > 0 &&
                        <Tab 
                            label={`Selected (${this.state.selectedImages.arr.length})`} 
                            onClick={this.foreignShowSelectedTab}
                        />
                        }

                    </Tabs>

                    { this.state.tabState === TABS_SEARCH &&
                        <ChipInput
                            newChipKeyCodes={[13, 32]}
                            dataSource={this.state.tags.map(t => t.name)}
                            label = "Search by tags"
                            className="center-field"
                            autoFocus = {true}
                            value = {this.state.searchTagNames}
                            onAdd={this.foreignAddTagToSearch}
                            onDelete={this.foreignRemoveTagFromSearch}
                        />
                    }

                    { this.state.tabState === TABS_SELECTED &&
                        <TagList 
                            className="center-field"
                            tags={selectedImageTags}
                            onAdd={(tagString) => this.addTagsToSelected(tagString)}
                            onRemove={(tag) => this.removeTagFromSelected(tag)}
                        />
                    }
                </Paper>

                <Drawer
                    variant="persistent"
                    open={this.state.isDrawerOpen}
                    >

                    <IconButton
                        onClick={() => this.mutateState(mut => mut.setDrawerOpenState(false))}
                        >

                        <Icon>chevron_left</Icon>
                    </IconButton>

                    <Divider />

                    <ListItem button
                        onClick={() => this.mutateState(mut => mut.setTagListDisplayState(!mut.getOldState().displayTagList))}
                        >
                        <ListItemIcon>
                            <Icon>
                                {this.state.displayTagList ? "expand_less" : "expand_more"}
                            </Icon>
                        </ListItemIcon>

                        <ListItemText primary="Tags"/>
                    </ListItem>

                    <Collapse 
                        in={this.state.displayTagList} 
                        timeout="auto" 
                        unmountOnExit 
                        style={{overflowY: "auto"}}
                        >

                        <List component="div" dense>
                            <ListItem>

                                <ListItemIcon>
                                    <Icon>sort</Icon>
                                </ListItemIcon>

                                <Select style={{width: "100%"}}
                                    value={this.state.tagOrdering.id}
                                    onChange={(e) => this.mutateState(mut => mut.setTagOrdering(e.target.value).orderTags())}
                                    inputProps={{
                                      name: 'order-by',
                                    }}
                                >
                                    {allOrderings.map(ord => <MenuItem key={ord.id} value={ord.id}>{ord.display}</MenuItem>)}
                                </Select>
                            </ListItem>

                            {this.state.tags.map(tag => {
                                
                                const isInSearch = this.state.searchTagNames.includes(tag.name);

                                return (
                                <ListItem dense key={tag.id}>
                                    
                                <ListItemAvatar>
                                        <Avatar>{tag.times_used}</Avatar>
                                    </ListItemAvatar>

                                    <ListItemText primary={tag.name}/>

                                    <ListItemSecondaryAction>

                                        { !isInSearch 
                                                ?

                                            <IconButton
                                                value={tag.name} 
                                                onClick={this.foreignAddTagButton}
                                                >
                                                <Icon>add</Icon>
                                            </IconButton>

                                                :

                                            <IconButton
                                                value={tag.name}
                                                onClick={this.foreignRemoveTagButton}
                                                >
                                                <Icon>remove</Icon>
                                            </IconButton>
                                        }
                                        
                                    </ListItemSecondaryAction>
                                    
                                </ListItem>
                            );
                            })}
                        </List>
                    </Collapse>

                    {this.state.displayTagList && <Divider />}

                    <ListItem button
                        onClick={() => this.mutateState(mut => mut.addAllInCurrentViewToSelection())}
                        >
                        <ListItemIcon>
                            <Icon>select_all</Icon>
                        </ListItemIcon>

                        <ListItemText primary="Select all"/>
                    </ListItem>


                    <ListItem button
                        onClick={() => this.mutateState(mut => mut.clearImgList(IMGS_SELECTED))}
                        >
                        <ListItemIcon>
                            <Icon>clear</Icon>
                        </ListItemIcon>

                        <ListItemText primary="Clear selection"/>
                    </ListItem>

                    <Divider />

                    <ListItem button
                        onClick={() => this.setDialogImageRowsState(true)}
                        >
                        <ListItemIcon>
                            <Icon>view_module</Icon>
                        </ListItemIcon>

                        <ListItemText primary="Adjust images per row"/>
                    </ListItem>

                    <ListItem button
                        onClick={() => this.state.api.reloadRoot().then(
                            () => this.mutateState(mut => mut.showSnackbar("Reloaded backend")))}
                        >
                        <ListItemIcon>
                            <Icon>refresh</Icon>
                        </ListItemIcon>

                        <ListItemText primary="Reload backend"/>
                    </ListItem>

                    <ListItem button
                        onClick={() => this.setDialogBackendUrlState(true)}
                        >
                        <ListItemIcon>
                            <Icon>build</Icon>
                        </ListItemIcon>

                        <ListItemText primary="Set backend URL"/>
                    </ListItem>

                    <Divider/>

                    <ListItem button
                        onClick={() => this.setDialogAboutState(true)}
                        >
                        <ListItemIcon>
                            <Icon>star</Icon>
                        </ListItemIcon>

                        <ListItemText primary="About"/>
                    </ListItem>

                </Drawer>

                <div className="image-list">
                    <Grid container spacing={16}>
                        {imageList}
                    </Grid>
                </div>

                {numPages > 0 && 
                <div>
                    {nextPageButtons}
                </div>
                }

                <Dialog
                    open={this.state.dialogAbout}
                    onClose={() => this.setDialogAboutState(false)}
                    >

                    <DialogTitle>{`Tifari web frontend v${VERSION}`}</DialogTitle>

                    <DialogContent>
                        <DialogContentText>
                            Built using <a href="https://github.com/facebook/react/">react</a>, <a href="https://github.com/mui-org/material-ui">material-ui</a>, <a href="https://github.com/TeamWertarbyte/material-ui-chip-input">material-ui-chip-input</a>
                        </DialogContentText>

                        <List dense>
                            <ListItem>

                                <ListItemIcon>
                                    <SvgIcon>
                                        <path d="M12.007 0C6.12 0 1.1 4.27.157 10.08c-.944 5.813 2.468 11.45 8.054 13.312.19.064.397.033.555-.084.16-.117.25-.304.244-.5v-2.042c-3.33.735-4.037-1.56-4.037-1.56-.22-.726-.694-1.35-1.334-1.756-1.096-.75.074-.735.074-.735.773.103 1.454.557 1.846 1.23.694 1.21 2.23 1.638 3.45.96.056-.61.327-1.178.766-1.605-2.67-.3-5.462-1.335-5.462-6.002-.02-1.193.42-2.35 1.23-3.226-.327-1.015-.27-2.116.166-3.09 0 0 1.006-.33 3.3 1.23 1.966-.538 4.04-.538 6.003 0 2.295-1.5 3.3-1.23 3.3-1.23.445 1.006.49 2.144.12 3.18.81.877 1.25 2.033 1.23 3.226 0 4.607-2.805 5.627-5.476 5.927.578.583.88 1.386.825 2.206v3.29c-.005.2.092.393.26.507.164.115.377.14.565.063 5.568-1.88 8.956-7.514 8.007-13.313C22.892 4.267 17.884.007 12.008 0z" />
                                    </SvgIcon>
                                </ListItemIcon>

                                <ListItemText>
                                    <a href="https://github.com/SSStormy/tifari">GitHub</a>
                                </ListItemText>

                            </ListItem>
                        </List>

                    </DialogContent>

                    <DialogActions>
                        <Button onClick={() => this.setDialogAboutState(false)} color="primary">
                            Close 
                        </Button>
                    </DialogActions>

                </Dialog>

                <Dialog
                    open={this.state.dialogBackendUrl}
                    onClose={() => this.setDialogBackendUrlState(false)}
                    >
                    <DialogTitle id="form-dialog-title">Set backend URL </DialogTitle>
                    <DialogContent>
                        <DialogContentText>
                            Tifari will use this URL to query searches, images, tags etc.
                        </DialogContentText>

                        <TextField
                            autoFocus
                            fullWidth
                            margin="dense"
                            id="url"
                            label="Backend URL"
                            type="url"
                            value = {this.state.backendUrlBuffer}
                            onChange={(ev) => { let val = ev.target.value; this.mutateState(mut => mut.setBackendUrlBuffer(val))}}
                        />

                        <DialogActions>
                            <Button onClick={() => this.setDialogBackendUrlState(false)} color="primary">
                                Cancel
                            </Button>
                            <Button onClick={() => { this.swapBackendUrl(); this.setDialogBackendUrlState(false);}} color="primary">
                                Set
                            </Button>
                        </DialogActions>

                    </DialogContent>
                </Dialog>

                <Dialog
                    open={this.state.dialogImageRowsOpen}
                    onClose={() => this.setDialogImageRowsState(false)}
                    >
                    <DialogTitle id="form-dialog-title">Adjust image rows</DialogTitle>
                    <DialogContent>

                        <Select
                            style={{width: "100%"}}
                            value={this.state.sliderCardSize}
                            onChange={(e) => this.mutateState(mut => mut.setSliderCardSize(e.target.value))}
                            inputProps={{
                              name: 'rows',
                            }}
                            >

                            {[1,2,3,4].map(i => <MenuItem key={i} value={i}>{i.toString()}</MenuItem>)}

                          </Select>
                        
                    </DialogContent>

                    <DialogActions>
                        <Button onClick={() => this.setDialogImageRowsState(false)} color="primary">
                         Close 
                        </Button>
                    </DialogActions>

                </Dialog>

                <Snackbar
                    open={this.state.showSnackbar}
                    anchorOrigin={{ vertical: "bottom", horizontal: "left" }}
                    onClose={() => this.mutateState(mut => mut.hideSnackbar())}
                    ContentProps={{
                        'aria-describedby': 'message-id',
                    }}
                    message={<span id="message-id">{this.state.snackbarMessage}</span>}
                />

            </React.Fragment>
        );
    }
}

export default App;
